use crate::api::handler::oauth_decisions::schema::OAuthDecisionAction;
use crate::models::convert::FromDb;
use crate::models::convert::IntoDb;

impl FromDb for OAuthDecisionAction {
    type DbFormat = super::OAuthDecisionAction;

    fn from_db(db_format: Self::DbFormat) -> Self {
        match db_format {
            super::OAuthDecisionAction::Accept => OAuthDecisionAction::Accept,
            super::OAuthDecisionAction::Deny => OAuthDecisionAction::Deny,
        }
    }
}
impl IntoDb for OAuthDecisionAction {
    fn into_db(self) -> Self::DbFormat {
        match self {
            OAuthDecisionAction::Accept => super::OAuthDecisionAction::Accept,
            OAuthDecisionAction::Deny => super::OAuthDecisionAction::Deny,
        }
    }
}
