use chrono::Utc;
use futures::TryStreamExt;
use log::debug;
use rorm::db::executor::Stream;
use rorm::db::sql::value::Value;
use rorm::db::transaction::Transaction;
use rorm::db::Executor;
use rorm::imr::Annotation;
use rorm::insert;
use rorm::prelude::ForeignModelByField;
use rorm::query;
use rorm::update;
use rorm::FieldAccess;
use rorm::Model;
use uuid::Uuid;

use crate::api::handler::attacks::schema::SimpleAttack;
use crate::api::handler::common::error::ApiError;
use crate::api::handler::common::error::ApiResult;
use crate::api::handler::users::schema::SimpleUser;
use crate::api::handler::workspaces::schema::FullWorkspace;
use crate::api::handler::workspaces::schema::SimpleWorkspace;
use crate::chan::global::GLOBAL;
use crate::chan::ws_manager::schema::WsMessage;
use crate::models::convert::FromDb;
use crate::models::Attack;
use crate::models::ModelType;
use crate::models::Search;
use crate::models::SearchResult;
use crate::models::User;
use crate::models::Workspace;
use crate::models::WorkspaceMember;

pub(crate) fn build_query_list() -> Vec<(String, ModelType)> {
    let table_names_no_ref_to_ws = vec![
        ModelType::DnsRecordResult,
        ModelType::DnsTxtScanResult,
        ModelType::DehashedQueryResult,
        ModelType::CertificateTransparencyResult,
        ModelType::HostAliveResult,
        ModelType::ServiceDetectionResult,
        ModelType::UdpServiceDetectionResult,
    ];

    let table_names_ref_to_ws = vec![
        ModelType::Host,
        ModelType::Service,
        ModelType::Port,
        ModelType::Domain,
    ];

    let mut data = Vec::with_capacity(table_names_no_ref_to_ws.len() + table_names_ref_to_ws.len());

    data.extend(table_names_no_ref_to_ws.into_iter().map(|table_entry| {
        (format!(
            "SELECT
                workspace_related_table.uuid
            FROM
                (SELECT t.* FROM \"{table_entry}\" t JOIN attack on t.attack = attack.uuid WHERE attack.workspace = $1) workspace_related_table
            WHERE
                (workspace_related_table.*)::text ILIKE $2;"
        ), table_entry)
    }).collect::<Vec<(String, ModelType)>>());

    data.extend(
        table_names_ref_to_ws
            .into_iter()
            .map(|table_entry| {
                (format!(
                    "SELECT
                            workspace_related_table.uuid
                        FROM
                            (SELECT t.* FROM \"{table_entry}\" t WHERE t.workspace = $1) workspace_related_table
                        WHERE
                            (workspace_related_table.*)::text ILIKE $2;"
                ),
                 table_entry,
                )
            })
            .collect::<Vec<(String, ModelType)>>(),
    );

    data
}

pub(crate) async fn run_search(
    search_term: &str,
    workspace_uuid: Uuid,
    search_uuid: Uuid,
    user_uuid: Uuid,
) -> Result<(), rorm::Error> {
    let search_term = format!("%{search_term}%");

    for (sql, model_type) in build_query_list() {
        debug!("search sql: {sql}");
        let mut stream = GLOBAL.db.execute::<Stream>(
            sql,
            vec![Value::Uuid(workspace_uuid), Value::String(&search_term)],
        );

        while let Some(row) = stream.try_next().await? {
            let ref_key: Uuid = row.get(0)?;

            debug!("received search result with key: {ref_key:?}");
            let result_uuid = insert!(&GLOBAL.db, SearchResult)
                .return_primary_key()
                .single(&SearchResult {
                    uuid: Uuid::new_v4(),
                    ref_key,
                    ref_type: model_type,
                    search: ForeignModelByField::Key(search_uuid),
                })
                .await?;

            GLOBAL
                .ws
                .message(
                    user_uuid,
                    WsMessage::SearchNotify {
                        search_uuid,
                        result_uuid,
                    },
                )
                .await;
        }
    }

    update!(&GLOBAL.db, Search)
        .condition(Search::F.uuid.equals(search_uuid))
        .set(Search::F.finished_at, Some(Utc::now()))
        .await?;

    Ok(())
}

/// Get a [`FullWorkspace`] by its uuid without permission checks
pub(crate) async fn get_workspace_unchecked(
    uuid: Uuid,
    tx: &mut Transaction,
) -> ApiResult<FullWorkspace> {
    let workspace = query!(&mut *tx, Workspace)
        .condition(Workspace::F.uuid.equals(uuid))
        .optional()
        .await?
        .ok_or(ApiError::InvalidUuid)?;

    let owner = query!(&mut *tx, SimpleUser)
        .condition(User::F.uuid.equals(*workspace.owner.key()))
        .one()
        .await?;

    let attacks = query!(
        &mut *tx,
        (
            Attack::F.uuid,
            Attack::F.attack_type,
            Attack::F.finished_at,
            Attack::F.created_at,
            Attack::F.started_by as SimpleUser,
            Attack::F.error,
        )
    )
    .condition(Attack::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(
        |(attack_uuid, attack_type, finished_at, created_at, started_by, error)| SimpleAttack {
            uuid: attack_uuid,
            workspace: SimpleWorkspace {
                uuid: workspace.uuid,
                name: workspace.name.clone(),
                description: workspace.description.clone(),
                created_at: workspace.created_at,
                owner: owner.clone(),
                archived: workspace.archived,
            },
            attack_type: FromDb::from_db(attack_type),
            started_by,
            finished_at,
            created_at,
            error,
        },
    )
    .collect();

    let members = query!(
        &mut *tx,
        (
            WorkspaceMember::F.member.uuid,
            WorkspaceMember::F.member.username,
            WorkspaceMember::F.member.display_name
        )
    )
    .condition(WorkspaceMember::F.workspace.equals(uuid))
    .all()
    .await?
    .into_iter()
    .map(|(uuid, username, display_name)| SimpleUser {
        uuid,
        username,
        display_name,
    })
    .collect();

    Ok(FullWorkspace {
        uuid: workspace.uuid,
        name: workspace.name,
        description: workspace.description,
        notes: GLOBAL
            .editor_cache
            .ws_notes
            .get(workspace.uuid)
            .await?
            .unwrap_or_default()
            .0,
        owner,
        attacks,
        members,
        archived: workspace.archived,
        created_at: workspace.created_at,
    })
}

/// Inserts a bulk of models skipping all fields which are not insertable
pub async fn insert_models<'ex, M: Model>(
    executor: impl Executor<'ex>,
    models: Vec<M>,
) -> Result<(), rorm::Error> {
    let mut columns = Vec::with_capacity(M::COLUMNS.len());
    let mut to_skip = Vec::new();
    'field: for (index, field) in M::get_imr().fields.into_iter().enumerate() {
        for annotation in field.annotations {
            match annotation {
                Annotation::AutoCreateTime
                | Annotation::AutoUpdateTime
                | Annotation::AutoIncrement => {
                    to_skip.push(index);
                    continue 'field;
                }
                _ => {}
            }
        }

        columns.push(M::COLUMNS[index]);
    }

    let mut tmp_values = Vec::with_capacity(M::COLUMNS.len());
    let mut values = Vec::with_capacity(columns.len() * models.len());
    for model in models {
        model.push_values(&mut tmp_values);
        for (index, value) in tmp_values.drain(..).enumerate() {
            if to_skip.contains(&index) {
                continue;
            }

            values.push(value);
        }
    }

    let sql_values = values.iter().map(|v| v.as_sql()).collect::<Vec<_>>();
    let rows = sql_values.chunks(columns.len()).collect::<Vec<_>>();

    rorm::db::database::insert_bulk(executor, M::TABLE, &columns, &rows).await?;

    Ok(())
}
