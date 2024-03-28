use std::str::FromStr;
use std::time::Duration;

use actix_web::web::Data;
use actix_web::web::Form;
use actix_web::web::Json;
use actix_web::web::Path;
use actix_web::web::Query;
use actix_web::web::Redirect;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use log::debug;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::thread_rng;
use rorm::query;
use rorm::FieldAccess;
use rorm::Model;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use uuid::Uuid;

use crate::api::extractors::SessionUser;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::schema::PathUuid;
use crate::api::handler::oauth::schema::OAuthDecisionQuery;
use crate::api::handler::oauth::schema::OpenRequestInfo;
use crate::api::handler::oauth::utils::build_redirect;
use crate::api::handler::oauth_applications::schema::SimpleOauthClient;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::models::OAuthDecision;
use crate::models::OAuthDecisionAction;
use crate::models::OauthClient;
use crate::models::Workspace;
use crate::models::WorkspaceAccessToken;
use crate::modules::oauth::schemas::AuthError;
use crate::modules::oauth::schemas::AuthErrorType;
use crate::modules::oauth::schemas::AuthRequest;
use crate::modules::oauth::schemas::CodeChallengeMethod;
use crate::modules::oauth::schemas::TokenError;
use crate::modules::oauth::schemas::TokenErrorType;
use crate::modules::oauth::schemas::TokenRequest;
use crate::modules::oauth::schemas::TokenResponse;
use crate::modules::oauth::OAuthRequest;
use crate::modules::oauth::OAuthScope;
use crate::modules::oauth::OauthManager;
use crate::modules::oauth::OpenIfError;

/// Initial endpoint an application redirects the user to.
///
/// It requires both the `state` parameter against CSRF, as well as a pkce challenge.
/// The only supported pkce `code_challenge_method` is `S256`.
#[swaggapi::get("/auth", tags("OAuth"))]
pub async fn auth(
    manager: Data<OauthManager>,
    request: Query<AuthRequest>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Redirect, ApiError> {
    let request = request.into_inner();

    let Some(client) = query!(&GLOBAL.db, OauthClient)
        .condition(OauthClient::F.uuid.equals(request.client_id))
        .optional()
        .await?
    else {
        return if let Some(redirect_uri) = request.redirect_uri.as_deref() {
            build_redirect(
                redirect_uri,
                AuthError {
                    error: AuthErrorType::UnauthorizedClient,
                    state: request.state,
                    error_description: Some("Unregistered client"),
                },
            )
        } else {
            Err(ApiError::InvalidUuid)
        };
    };

    if request.response_type != "code" {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::UnsupportedResponseType,
                state: request.state,
                error_description: Some("Only supported response_type is code"),
            },
        );
    }
    if let Some(redirect_uri) = request.redirect_uri {
        if redirect_uri != client.redirect_uri {
            return build_redirect(
                &client.redirect_uri,
                AuthError {
                    error: AuthErrorType::InvalidRequest,
                    state: request.state,
                    error_description: Some("Invalid redirect_uri"),
                },
            );
        }
    }

    let Some(raw_scope) = request.scope else {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::InvalidScope,
                state: request.state,
                error_description: Some("Missing scope"),
            },
        );
    };

    let Some(Ok(workspace)) = raw_scope.strip_prefix("workspace/").map(Uuid::from_str) else {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::InvalidScope,
                state: request.state,
                error_description: Some("Couldn't parse scope"),
            },
        );
    };

    let Some(state) = request.state else {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::InvalidRequest,
                state: None,
                error_description: Some("Missing state"),
            },
        );
    };

    let Some(code_challenge) = request.code_challenge else {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::InvalidRequest,
                state: None,
                error_description: Some("Missing code_challenge"),
            },
        );
    };
    if !matches!(request.code_challenge_method, CodeChallengeMethod::Sha256) {
        return build_redirect(
            &client.redirect_uri,
            AuthError {
                error: AuthErrorType::InvalidRequest,
                state: None,
                error_description: Some("Unsupported code_challenge_method"),
            },
        );
    }

    let open_request = OAuthRequest {
        client_pk: request.client_id,
        state,
        scope: OAuthScope { workspace },
        user: user_uuid,
        code_challenge,
    };

    if let Some(action) =
        OAuthDecision::get(&GLOBAL.db, user_uuid, request.client_id, open_request.scope).await?
    {
        match action {
            OAuthDecisionAction::Accept => {
                let state = open_request.state.clone();
                let code = manager.insert_accepted(open_request);
                #[derive(Serialize, Debug)]
                struct AcceptRedirect {
                    code: Uuid,
                    state: String,
                }
                build_redirect(&client.redirect_uri, AcceptRedirect { code, state })
            }
            OAuthDecisionAction::Deny => build_redirect(
                &client.redirect_uri,
                AuthError {
                    error: AuthErrorType::AccessDenied,
                    state: Some(open_request.state),
                    error_description: None,
                },
            ),
        }
    } else {
        let request_uuid = manager.insert_open(open_request);
        Ok(Redirect::to(format!("/#/oauth-request/{request_uuid}")))
    }
}

/// Queried by the frontend to display information about the oauth request to the user
#[swaggapi::get("/info/{uuid}", tags("OAuth"))]
pub async fn info(
    path: Path<PathUuid>,
    manager: Data<OauthManager>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Json<OpenRequestInfo>, ApiError> {
    let request = manager.get_open(path.uuid).ok_or(ApiError::InvalidUuid)?;

    if user_uuid != request.user {
        return Err(ApiError::MissingPrivileges);
    }

    let oauth_application = query!(&GLOBAL.db, SimpleOauthClient)
        .condition(OauthClient::F.uuid.equals(request.client_pk))
        .one()
        .await?;
    let (uuid, name, description, created_at, owner, archived) = query!(
        &GLOBAL.db,
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.owner,
            Workspace::F.archived,
        )
    )
    .condition(Workspace::F.uuid.equals(request.scope.workspace))
    .one()
    .await?;

    let owner = GLOBAL
        .user_cache
        .get_simple_user(*owner.key())
        .await?
        .ok_or(ApiError::InternalServerError)?;

    Ok(Json(OpenRequestInfo {
        workspace: SimpleWorkspace {
            uuid,
            name,
            description,
            created_at,
            owner,
            archived,
        },
        oauth_application,
    }))
}

/// Endpoint visited by user to grant a requesting application access
#[swaggapi::get("/accept/{uuid}", tags("OAuth"))]
pub async fn accept(
    path: Path<PathUuid>,
    manager: Data<OauthManager>,
    SessionUser(user_uuid): SessionUser,
    query: Query<OAuthDecisionQuery>,
) -> Result<Redirect, ApiError> {
    let open_request =
        match manager.remove_open_if(path.uuid, |open_request| open_request.user == user_uuid) {
            Ok(open_request) => open_request,
            Err(OpenIfError::NotFound) => return Err(ApiError::InvalidUuid),
            Err(OpenIfError::FailedCheck) => return Err(ApiError::MissingPrivileges),
        };
    let code = manager.insert_accepted(open_request.clone());

    // Remember decision
    if query.remember {
        OAuthDecision::insert(
            &GLOBAL.db,
            user_uuid,
            open_request.client_pk,
            open_request.scope,
            OAuthDecisionAction::Accept,
        )
        .await?;
    }

    // Redirect
    let (redirect_uri,) = query!(&GLOBAL.db, (OauthClient::F.redirect_uri,))
        .condition(OauthClient::F.uuid.equals(open_request.client_pk))
        .one()
        .await?;
    #[derive(Serialize, Debug)]
    struct AcceptRedirect {
        code: Uuid,
        state: String,
    }

    build_redirect(
        &redirect_uri,
        AcceptRedirect {
            code,
            state: open_request.state,
        },
    )
}

/// Endpoint visited by user to deny a requesting application access
#[swaggapi::get("/deny/{uuid}", tags("OAuth"))]
pub async fn deny(
    manager: Data<OauthManager>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
    query: Query<OAuthDecisionQuery>,
) -> Result<Redirect, ApiError> {
    let open_request =
        match manager.remove_open_if(path.uuid, |open_request| open_request.user == user_uuid) {
            Ok(open_request) => open_request,
            Err(OpenIfError::NotFound) => return Err(ApiError::InvalidUuid),
            Err(OpenIfError::FailedCheck) => return Err(ApiError::MissingPrivileges),
        };

    // Remember decision
    if query.remember {
        OAuthDecision::insert(
            &GLOBAL.db,
            user_uuid,
            open_request.client_pk,
            open_request.scope,
            OAuthDecisionAction::Deny,
        )
        .await?;
    }

    // Redirect
    let (redirect_uri,) = query!(&GLOBAL.db, (OauthClient::F.redirect_uri,))
        .condition(OauthClient::F.uuid.equals(open_request.client_pk))
        .one()
        .await?;
    build_redirect(
        &redirect_uri,
        AuthError {
            error: AuthErrorType::AccessDenied,
            state: Some(open_request.state),
            error_description: None,
        },
    )
}

/// Endpoint an application calls itself after the user accepted and was redirected back to it.
#[swaggapi::post("/token", tags("OAuth"))]
pub async fn token(
    manager: Data<OauthManager>,
    request: Form<TokenRequest>,
) -> Result<Json<TokenResponse>, TokenError> {
    let TokenRequest {
        grant_type,
        code,
        redirect_uri,
        client_id,
        client_secret,
        code_verifier,
    } = request.into_inner();

    if grant_type != "authorization_code" {
        return Err(TokenError {
            error: TokenErrorType::UnsupportedGrantType,
            error_description: Some("Only supported response_type is authorization_code"),
        });
    }

    let accepted = manager.get_accepted(code).ok_or(TokenError {
        error: TokenErrorType::InvalidRequest,
        error_description: Some("Invalid code"),
    })?;
    let client = query!(&GLOBAL.db, OauthClient)
        .condition(OauthClient::F.uuid.equals(accepted.client_pk))
        .one()
        .await?;

    let verifier = code_verifier.ok_or(TokenError {
        error: TokenErrorType::InvalidRequest,
        error_description: Some("Missing code_verifier"),
    })?;
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let computed = BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize());

    if accepted.code_challenge != computed {
        debug!(
            "PKCE failed; computed: {computed}, should be: {challenge}",
            challenge = accepted.code_challenge
        );
        return Err(TokenError {
            error: TokenErrorType::InvalidRequest,
            error_description: Some("Missing code_verifier doesn't match code_challenge"),
        });
    }

    if client_id != client.uuid
        || client_secret != client.secret
        || redirect_uri != Some(client.redirect_uri)
    {
        return Err(TokenError {
            error: TokenErrorType::InvalidClient,
            error_description: Some("Invalid client_id, client_secret or redirect_uri"),
        });
    }

    let access_token = Alphanumeric.sample_string(&mut thread_rng(), 32);
    let expires_in = Duration::from_secs(120);

    WorkspaceAccessToken::insert(
        &GLOBAL.db,
        access_token.clone(),
        Utc::now() + expires_in,
        accepted.user,
        accepted.scope.workspace,
        client.uuid,
    )
    .await?;

    Ok(Json(TokenResponse {
        token_type: "access_token",
        access_token,
        expires_in,
    }))
}
