use rorm::db::Executor;
use rorm::prelude::*;
use rorm::{and, insert, query};
use uuid::Uuid;

use crate::models::{OAuthDecision, OAuthDecisionAction};
use crate::modules::oauth::OAuthScope;

impl OAuthDecision {
    /// Insert a new [`OAuthDecision`]
    pub async fn insert(
        executor: impl Executor<'_>,
        user: Uuid,
        app: Uuid,
        scope: OAuthScope,
        action: OAuthDecisionAction,
    ) -> Result<Uuid, rorm::Error> {
        let OAuthScope { workspace } = scope;
        insert!(executor, OAuthDecision)
            .return_primary_key()
            .single(&OAuthDecision {
                uuid: Uuid::new_v4(),
                user: ForeignModelByField::Key(user),
                app: ForeignModelByField::Key(app),
                scope_workspace: workspace,
                action,
            })
            .await
    }

    /// Get a [`OAuthDecision`]'s action (if it exists)
    pub async fn get(
        executor: impl Executor<'_>,
        user: Uuid,
        app: Uuid,
        scope: OAuthScope,
    ) -> Result<Option<OAuthDecisionAction>, rorm::Error> {
        let OAuthScope { workspace } = scope;
        let option = query!(executor, (OAuthDecision::F.action,))
            .condition(and![
                OAuthDecision::F.app.equals(app),
                OAuthDecision::F.user.equals(user),
                OAuthDecision::F.scope_workspace.equals(workspace)
            ])
            .optional()
            .await?;
        Ok(option.map(|(action,)| action))
    }
}
