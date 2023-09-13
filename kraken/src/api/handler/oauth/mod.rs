// TODO eliminate non-500 api errors

mod applications;
mod schemas;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;

use actix_web::web::{Data, Form, Json, Path, Query, Redirect};
use actix_web::{get, post};
use log::error;
use rorm::prelude::*;
use rorm::{query, Database};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

pub use self::applications::*;
use self::schemas::*;
use crate::api::handler::{ApiError, PathUuid, SessionUser, SimpleWorkspace, UserResponse};
use crate::models::{OauthClient, User, Workspace};

#[derive(Debug, Default)]
pub(crate) struct OauthManager(Mutex<OauthManagerInner>);
#[derive(Debug, Default)]
struct OauthManagerInner {
    /// Waiting for user interaction i.e. `/accept` or `/deny`
    ///
    /// Uses a `uuid` as key which is presented to the user's agent
    open: HashMap<Uuid, OpenRequest>,

    /// Waiting for server interaction i.e. `/token`
    ///
    /// Uses `code` as key which is passed through the user's agent to the client
    accepted: HashMap<Uuid, OpenRequest>,
}

impl OauthManager {
    fn insert_open(&self, request: OpenRequest) -> Uuid {
        let mut inner = self.0.lock().unwrap();
        loop {
            let uuid = Uuid::new_v4();
            if let Entry::Vacant(entry) = inner.open.entry(uuid) {
                entry.insert(request);
                return uuid;
            }
        }
    }
}

/// Open oauth request which is waiting for user interactions
#[derive(Debug, Clone)]
struct OpenRequest {
    /// Pk of the requesting [`OauthClient`]
    client_pk: Uuid,

    /// State provided by client in `/auth`
    state: Option<String>,

    /// Scope requested by client
    scope: Scope,

    /// User which is being asked
    user: Uuid,
}

#[derive(Debug, Clone)]
struct Scope {
    workspace: Uuid,
}

/// Initial endpoint an application redirects the user to
#[get("/auth")]
pub(crate) async fn auth(
    db: Data<Database>,
    manager: Data<OauthManager>,
    request: Query<AuthRequest>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Redirect, ApiError> {
    let request = request.into_inner();

    let client = query!(db.as_ref(), OauthClient)
        .condition(OauthClient::F.uuid.equals(request.client_id))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?; // TODO redirect unauthorized_client

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

    let request_uuid = manager.insert_open(OpenRequest {
        client_pk: request.client_id,
        state: request.state,
        scope: Scope { workspace },
        user: user_uuid,
    });

    Ok(Redirect::to(format!("/#/oauth-request/{request_uuid}")))
}

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
pub(crate) async fn info(
    db: Data<Database>,
    path: Path<PathUuid>,
    manager: Data<OauthManager>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Json<OpenRequestInfo>, ApiError> {
    let request = {
        let inner = manager.0.lock().unwrap();
        inner
            .open
            .get(&path.uuid)
            .ok_or(ApiError::InvalidUuid)?
            .clone()
    };

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
            owner: UserResponse {
                display_name: owner.display_name,
                username: owner.username,
                uuid: owner.uuid,
            },
        },
        oauth_application,
    }))
}

/// Endpoint visited by user to grant a requesting application access
#[get("/accept/{uuid}")]
pub(crate) async fn accept(
    db: Data<Database>,
    path: Path<PathUuid>,
    manager: Data<OauthManager>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Redirect, ApiError> {
    let open_request;
    let code;
    {
        let mut inner = manager.0.lock().unwrap();

        // Check validity
        open_request = inner.open.remove(&path.uuid).ok_or(ApiError::InvalidUuid)?;
        if open_request.user != user_uuid {
            inner.open.insert(path.uuid, open_request);
            return Err(ApiError::MissingPrivileges);
        }

        // Advance request
        code = Uuid::new_v4();
        inner.accepted.insert(code, open_request.clone());
    };

    // Redirect
    let (redirect_uri,) = query!(db.as_ref(), (OauthClient::F.redirect_uri,))
        .condition(OauthClient::F.uuid.equals(open_request.client_pk))
        .one()
        .await?;
    #[derive(Serialize, Debug)]
    struct AcceptRedirect {
        code: Uuid,
        state: Option<String>,
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
#[get("/deny/{uuid}")]
pub(crate) async fn deny(
    db: Data<Database>,
    manager: Data<OauthManager>,
    path: Path<PathUuid>,
    SessionUser(user_uuid): SessionUser,
) -> Result<Redirect, ApiError> {
    let open_request = {
        let mut inner = manager.0.lock().unwrap();

        // Check validity
        let open_request = inner.open.remove(&path.uuid).ok_or(ApiError::InvalidUuid)?;
        if open_request.user != user_uuid {
            inner.open.insert(path.uuid, open_request);
            return Err(ApiError::MissingPrivileges);
        }

        open_request
    };

    // Redirect
    let (redirect_uri,) = query!(db.as_ref(), (OauthClient::F.redirect_uri,))
        .condition(OauthClient::F.uuid.equals(open_request.client_pk))
        .one()
        .await?;
    build_redirect(
        &redirect_uri,
        AuthError {
            error: AuthErrorType::AccessDenied,
            state: open_request.state,
            error_description: None,
        },
    )
}

/// Endpoint an application calls itself after the user accepted and was redirected back to it.
#[post("/token")]
pub(crate) async fn token(
    db: Data<Database>,
    manager: Data<OauthManager>,
    request: Form<TokenRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let TokenRequest {
        grant_type: _grant_type, // "handled" by serde
        code,
        redirect_uri,
        client_id,
        client_secret,
    } = request.into_inner();

    let accepted = {
        let inner = manager.0.lock().unwrap();
        inner
            .accepted
            .get(&code)
            .ok_or(ApiError::InvalidUuid)?
            .clone()
    };
    let client = query!(db.as_ref(), OauthClient)
        .condition(OauthClient::F.uuid.equals(accepted.client_pk))
        .one()
        .await?;

    if client_id != client.uuid {
        todo!();
    }
    if client_secret != client.secret {
        todo!();
    }
    if redirect_uri != client.redirect_uri {
        todo!();
    }

    // TODO: generate properly and store in db
    let access_token = "Very secure".to_string();
    let expires_in = Duration::from_secs(60);

    Ok(Json(TokenResponse {
        access_token,
        expires_in,
    }))
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
