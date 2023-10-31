//! OAuth related code lives here

use std::str::FromStr;
use std::time::Duration;

use actix_web::body::BoxBody;
use actix_web::web::{Data, Form, Json, Path, Query, Redirect};
use actix_web::{get, post, HttpResponse, ResponseError};
use base64::prelude::*;
use chrono::Utc;
use log::{debug, error};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use rorm::prelude::*;
use rorm::{and, delete, query, Database};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::api::extractors::SessionUser;
use crate::api::handler::oauth_applications::SimpleOauthClient;
use crate::api::handler::users::SimpleUser;
use crate::api::handler::workspaces::SimpleWorkspace;
use crate::api::handler::{ApiError, PathUuid};
use crate::models::{
    OAuthDecision, OAuthDecisionAction, OauthClient, User, Workspace, WorkspaceAccessToken,
};
use crate::modules::oauth::schemas::{
    AuthError, AuthErrorType, AuthRequest, CodeChallengeMethod, TokenError, TokenErrorType,
    TokenRequest, TokenResponse,
};
use crate::modules::oauth::{OAuthRequest, OAuthScope, OauthManager, OpenIfError};

/// Initial endpoint an application redirects the user to.
///
/// It requires both the `state` parameter against CSRF, as well as a pkce challenge.
/// The only supported pkce `code_challenge_method` is `S256`.
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth",
    responses(
        (status = 302, description = "The user is redirected to the frontend"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(AuthRequest),
    security(("api_key" = []))
)]
#[get("/auth")]
pub async fn auth(
    db: Data<Database>,
    manager: Data<OauthManager>,
    request: Query<AuthRequest>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Redirect, ApiError> {
    let request = request.into_inner();

    let Some(client) = query!(db.as_ref(), OauthClient)
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

    if let Some(action) = OAuthDecision::get(
        db.as_ref(),
        user_uuid,
        request.client_id,
        open_request.scope,
    )
    .await?
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

/// The information about an oauth request
#[derive(Serialize, ToSchema)]
pub struct OpenRequestInfo {
    pub(crate) workspace: SimpleWorkspace,
    pub(crate) oauth_application: SimpleOauthClient,
}

/// Queried by the frontend to display information about the oauth request to the user
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth",
    responses(
        (status = 200, description = "Return information about an ongoing oauth request", body = OpenRequestInfo),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid),
    security(("api_key" = []))
)]
#[get("/info/{uuid}")]
pub async fn info(
    db: Data<Database>,
    path: Path<PathUuid>,
    manager: Data<OauthManager>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Json<OpenRequestInfo>, ApiError> {
    let request = manager.get_open(path.uuid).ok_or(ApiError::InvalidUuid)?;

    if user_uuid != request.user {
        return Err(ApiError::MissingPrivileges);
    }

    let oauth_application = query!(db.as_ref(), SimpleOauthClient)
        .condition(OauthClient::F.uuid.equals(request.client_pk))
        .one()
        .await?;
    let (uuid, name, description, created_at, owner) = query!(
        db.as_ref(),
        (
            Workspace::F.uuid,
            Workspace::F.name,
            Workspace::F.description,
            Workspace::F.created_at,
            Workspace::F.owner,
        )
    )
    .condition(Workspace::F.uuid.equals(request.scope.workspace))
    .one()
    .await?;

    let owner = query!(db.as_ref(), User)
        .condition(User::F.uuid.equals(*owner.key()))
        .one()
        .await?;

    Ok(Json(OpenRequestInfo {
        workspace: SimpleWorkspace {
            uuid,
            name,
            description,
            created_at,
            owner: SimpleUser {
                display_name: owner.display_name,
                username: owner.username,
                uuid: owner.uuid,
            },
        },
        oauth_application,
    }))
}

/// Query parameters for `/accept` and `/deny`
#[derive(Deserialize, IntoParams)]
pub struct OAuthDecisionQuery {
    /// Should kraken remember this decision?
    #[serde(default)]
    pub remember: bool,
}

/// Endpoint visited by user to grant a requesting application access
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth",
    responses(
        (status = 302, description = "The user is redirected back to the requesting client"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, OAuthDecisionQuery),
    security(("api_key" = []))
)]
#[get("/accept/{uuid}")]
pub async fn accept(
    db: Data<Database>,
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
            db.as_ref(),
            user_uuid,
            open_request.client_pk,
            open_request.scope,
            OAuthDecisionAction::Accept,
        )
        .await?;
    }

    // Redirect
    let (redirect_uri,) = query!(db.as_ref(), (OauthClient::F.redirect_uri,))
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
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth",
    responses(
        (status = 302, description = "The user is redirected back to the requesting client"),
        (status = 400, description = "Client error", body = ApiErrorResponse),
        (status = 500, description = "Server error", body = ApiErrorResponse),
    ),
    params(PathUuid, OAuthDecisionQuery),
    security(("api_key" = []))
)]
#[get("/deny/{uuid}")]
pub async fn deny(
    db: Data<Database>,
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
            db.as_ref(),
            user_uuid,
            open_request.client_pk,
            open_request.scope,
            OAuthDecisionAction::Deny,
        )
        .await?;
    }

    // Redirect
    let (redirect_uri,) = query!(db.as_ref(), (OauthClient::F.redirect_uri,))
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
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth-server",
    responses(
        (status = 302, description = "Got token", body = TokenResponse),
        (status = 400, description = "Client error", body = TokenError),
        (status = 500, description = "Server error", body = TokenError),
    ),
    request_body = TokenRequest,
)]
#[post("/token")]
pub async fn token(
    db: Data<Database>,
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
    let client = query!(db.as_ref(), OauthClient)
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
        db.as_ref(),
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

/// Request to revoke a token
#[derive(Deserialize, ToSchema)]
pub struct RevokeTokenRequest {
    /// The token to be revoked.
    pub token: String,

    /// The client identifier
    pub client_id: Uuid,

    /// The client's secret to authenticate itself
    pub client_secret: String,
}

/// Endpoint an application calls itself to revoke an access token
#[utoipa::path(
    tag = "OAuth",
    context_path = "/api/v1/oauth-server",
    responses(
        (status = 200, description = "Token was revoked"),
        (status = 400, description = "Client error", body = TokenError),
        (status = 500, description = "Server error", body = TokenError),
    ),
    request_body = RevokeTokenRequest,
)]
#[post("/revoke")]
pub async fn revoke(
    db: Data<Database>,
    request: Form<RevokeTokenRequest>,
) -> Result<HttpResponse, TokenError> {
    let request = request.into_inner();
    let token_str: &str = request.token.as_ref();

    let mut tx = db.start_transaction().await?;

    if query!(&mut tx, OauthClient)
        .condition(and!(
            (OauthClient::F.uuid.equals(request.client_id)),
            (OauthClient::F.secret.equals(request.client_secret))
        ))
        .optional()
        .await?
        .is_none()
    {
        return Err(TokenError {
            error: TokenErrorType::InvalidClient,
            error_description: Some("Invalid client_id or client_secret"),
        });
    }

    if query!(&mut tx, WorkspaceAccessToken)
        .condition(WorkspaceAccessToken::F.token.equals(token_str))
        .optional()
        .await?
        .is_none()
    {
        return Err(TokenError {
            error: TokenErrorType::InvalidRequest,
            error_description: Some("Invalid token"),
        });
    }

    delete!(&mut tx, WorkspaceAccessToken)
        .condition(WorkspaceAccessToken::F.token.equals(token_str))
        .await?;

    Ok(HttpResponse::Ok().finish())
}

fn build_redirect(
    url: &str,
    query: impl Serialize + std::fmt::Debug,
) -> Result<Redirect, ApiError> {
    let Ok(mut url) = Url::parse(url) else {
        error!("Failed to parse url: {url}");
        return Err(ApiError::InternalServerError);
    };

    {
        let mut pairs = url.query_pairs_mut();
        let serializer = serde_urlencoded::Serializer::new(&mut pairs);
        if query.serialize(serializer).is_err() {
            error!("Failed to serialize url query: {query:?}");
            return Err(ApiError::InternalServerError);
        }
    }

    Ok(Redirect::to(url.to_string()))
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error_description {
            Some(desc) => write!(f, "{:?}: {desc}", self.error),
            None => write!(f, "{:?}", self.error),
        }
    }
}

impl From<rorm::Error> for TokenError {
    fn from(err: rorm::Error) -> Self {
        error!("Database error in `/token` endpoint: {err}");
        Self {
            error: TokenErrorType::ServerError,
            error_description: Some("An internal server error occured"),
        }
    }
}

impl ResponseError for TokenError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        (match self.error {
            TokenErrorType::ServerError => HttpResponse::InternalServerError(),
            TokenErrorType::InvalidClient => HttpResponse::Unauthorized(),
            _ => HttpResponse::BadRequest(),
        })
        .json(self)
    }
}
