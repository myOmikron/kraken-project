use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::prelude::*;

use crate::models::{Port, Service};
use crate::modules::raw_query::RawJoin;

/// Joins ports to the services table
pub struct JoinPorts;

impl<'a> RawJoin<'a> for JoinPorts {
    fn append(self, sql: &mut String, _values: &mut Vec<Value<'a>>) -> fmt::Result {
        const SERVICE: &str = Service::TABLE;
        const SERVICE_PORT: &str = <field!(Service::F.port)>::NAME;
        const PORT: &str = Port::TABLE;
        const PORT_UUID: &str = <field!(Port::F.uuid)>::NAME;
        write!(
            sql,
            r#"LEFT JOIN "{PORT}" ON "{SERVICE}"."{SERVICE_PORT}" = "{PORT}"."{PORT_UUID}""#
        )
    }
}

#[derive(Copy, Clone)]
pub struct JoinTags {
    target: &'static str,
    target_uuid: &'static str,

    workspacetag: &'static str,
    workspacetag_uuid: &'static str,
    workspacetag_name: &'static str,

    globaltag: &'static str,
    globaltag_uuid: &'static str,
    globaltag_name: &'static str,

    m2m_workspacetag: &'static str,
    m2m_workspacetag_workspacetag: &'static str,
    m2m_workspacetag_target: &'static str,

    m2m_globaltag: &'static str,
    m2m_globaltag_globaltag: &'static str,
    m2m_globaltag_target: &'static str,
}

/// Constructs a [`JoinTags`] instance
///
/// ## Example
/// ```ignore
/// join_tags!(Port, w: PortWorkspaceTag::F.port, g: PortGlobalTag::F.port)
/// ```
macro_rules! join_tags {
    ($TargetModel:ident, w: $WorkspaceModel:ident::F.$workspace_field:ident, g: $GlobalModel:ident::F.$global_field:ident) => {{
        use rorm::internal::field::Field;
        use rorm::{field, Model};
        use $crate::models::{
            $GlobalModel, $TargetModel, $WorkspaceModel, GlobalTag, WorkspaceTag,
        };

        $crate::modules::syntax::sqler::joins::JoinTags {
            target: $TargetModel::TABLE,
            target_uuid: <field!($TargetModel::F.uuid)>::NAME,

            workspacetag: WorkspaceTag::TABLE,
            workspacetag_uuid: <field!(WorkspaceTag::F.uuid)>::NAME,
            workspacetag_name: <field!(WorkspaceTag::F.name)>::NAME,

            globaltag: GlobalTag::TABLE,
            globaltag_uuid: <field!(GlobalTag::F.uuid)>::NAME,
            globaltag_name: <field!(GlobalTag::F.name)>::NAME,

            m2m_workspacetag: $WorkspaceModel::TABLE,
            m2m_workspacetag_workspacetag: <field!($WorkspaceModel::F.workspace_tag)>::NAME,
            m2m_workspacetag_target: <field!($WorkspaceModel::F.$workspace_field)>::NAME,

            m2m_globaltag: $GlobalModel::TABLE,
            m2m_globaltag_globaltag: <field!($GlobalModel::F.global_tag)>::NAME,
            m2m_globaltag_target: <field!($WorkspaceModel::F.$global_field)>::NAME,
        }
    }};
}

impl JoinTags {
    pub fn domain() -> Self {
        join_tags!(Domain, w: DomainWorkspaceTag::F.domain, g: DomainGlobalTag::F.domain)
    }
    pub fn host() -> Self {
        join_tags!(Host, w: HostWorkspaceTag::F.host, g: HostGlobalTag::F.host)
    }
    pub fn port() -> Self {
        join_tags!(Port, w: PortWorkspaceTag::F.port, g: PortGlobalTag::F.port)
    }
    pub fn service() -> Self {
        join_tags!(Service, w: ServiceWorkspaceTag::F.service, g: ServiceGlobalTag::F.service)
    }
}

impl<'a> RawJoin<'a> for JoinTags {
    fn append(self, sql: &mut String, _values: &mut Vec<Value<'a>>) -> fmt::Result {
        let Self {
            target,
            target_uuid,
            workspacetag,
            workspacetag_uuid,
            workspacetag_name,
            globaltag,
            globaltag_uuid,
            globaltag_name,
            m2m_workspacetag,
            m2m_workspacetag_workspacetag,
            m2m_workspacetag_target,
            m2m_globaltag,
            m2m_globaltag_globaltag,
            m2m_globaltag_target,
        } = self;
        write!(
            sql,
            r#" JOIN (
                SELECT
                    "{target}"."{target_uuid}",
                    array_agg("{workspacetag}"."{workspacetag_name}") || array_agg("{globaltag}"."{globaltag_name}") AS "tags"
                FROM
                    "{target}"
                        LEFT JOIN "{m2m_workspacetag}" ON "{target}"."{target_uuid}" = "{m2m_workspacetag}"."{m2m_workspacetag_target}"
                        LEFT JOIN "{workspacetag}" ON "{m2m_workspacetag}"."{m2m_workspacetag_workspacetag}" = "{workspacetag}"."{workspacetag_uuid}"
                        LEFT JOIN "{m2m_globaltag}" ON "{target}"."{target_uuid}" = "{m2m_globaltag}"."{m2m_globaltag_target}"
                        LEFT JOIN "{globaltag}" ON "{m2m_globaltag}"."{m2m_globaltag_globaltag}" = "{globaltag}"."{globaltag_uuid}"
                GROUP BY
                    "{target}"."{target_uuid}"
            ) AS "tags" ON "{target}"."{target_uuid}" = "tags"."{target_uuid}""#,
        )
    }
}
