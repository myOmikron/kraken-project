use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::handler::workspaces::schema::SimpleWorkspace;

/// Response holding a user's oauth decisions
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ListOauthDecisions {
    /// A user's oauth decisions
    pub decisions: Vec<FullOauthDecision>,
}

/// A user's remembered oauth decision
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct FullOauthDecision {
    /// The primary key
    pub uuid: Uuid,

    /// The application the decision was made for
    pub app: String,

    /// The requested workspace
    pub workspace: SimpleWorkspace,

    /// Action what to do with new incoming oauth requests
    #[schema(inline)]
    pub action: OAuthDecisionAction,
}

/// Action what to do with new oauth requests
#[derive(ToSchema, Serialize, Deserialize, Debug, Clone, Copy)]
pub enum OAuthDecisionAction {
    /// Auto accept new requests
    Accept,
    /// Auto deny new requests
    Deny,
}
