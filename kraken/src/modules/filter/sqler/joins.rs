use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::prelude::*;

use crate::models::Domain;
use crate::models::Host;
use crate::models::HttpService;
use crate::models::Port;
use crate::models::Service;
use crate::modules::raw_query::RawJoin;

macro_rules! sql_name {
    ($model:ident) => {
        <$model as Model>::TABLE as &'static str
    };
    ($model:ident::F.$field:ident) => {
        <field!($model::F.$field)>::NAME as &'static str
    };
}

pub fn from_port_join_host() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let port = sql_name!(Port);
        let port_host = sql_name!(Port::F.host);

        let host = sql_name!(Host);
        let host_uuid = sql_name!(Host::F.uuid);

        write!(
            sql,
            r#"LEFT JOIN "{host}" ON "{port}"."{port_host}" = "{host}"."{host_uuid}""#
        )
    }
}
pub fn from_service_join_port() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let service = sql_name!(Service);
        let service_port = sql_name!(Service::F.port);

        let port = sql_name!(Port);
        let port_uuid = sql_name!(Port::F.uuid);

        write!(
            sql,
            r#"LEFT JOIN "{port}" ON "{service}"."{service_port}" = "{port}"."{port_uuid}""#
        )
    }
}
pub fn from_service_join_host() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let service = sql_name!(Service);
        let service_host = sql_name!(Service::F.host);

        let host = sql_name!(Host);
        let host_uuid = sql_name!(Host::F.uuid);

        write!(
            sql,
            r#"JOIN "{host}" ON "{service}"."{service_host}" = "{host}"."{host_uuid}""#
        )
    }
}

pub fn from_http_service_join_port() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let service = sql_name!(HttpService);
        let service_port = sql_name!(HttpService::F.port);

        let port = sql_name!(Port);
        let port_uuid = sql_name!(Port::F.uuid);

        write!(
            sql,
            r#"LEFT JOIN "{port}" ON "{service}"."{service_port}" = "{port}"."{port_uuid}""#
        )
    }
}
pub fn from_http_service_join_host() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let service = sql_name!(HttpService);
        let service_host = sql_name!(HttpService::F.host);

        let host = sql_name!(Host);
        let host_uuid = sql_name!(Host::F.uuid);

        write!(
            sql,
            r#"JOIN "{host}" ON "{service}"."{service_host}" = "{host}"."{host_uuid}""#
        )
    }
}
pub fn from_http_service_join_domain() -> impl for<'a> RawJoin<'a> {
    |sql: &mut String, _: &mut Vec<Value>| {
        let http_service = sql_name!(HttpService);
        let http_service_domain = sql_name!(HttpService::F.domain);

        let domain = sql_name!(Domain);
        let domain_uuid = sql_name!(Domain::F.uuid);

        write!(
            sql,
            r#"JOIN "{domain}" ON "{http_service}"."{http_service_domain}" = "{domain}"."{domain_uuid}""#
        )
    }
}

/// Joins a subquery which retrieves an object's tags in a postgres array.
///
/// I.e. after applying this join, there is a column `"tags"."tags"` whose type is an array of strings.
///
/// Use the constructors [`JoinTags::domain`], [`JoinTags::host`], [`JoinTags::port`] and [`JoinTags::service`]
/// to select the base table this join is applied to.
#[derive(Copy, Clone)]
pub struct JoinTags {
    table_alias: &'static str,

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

macro_rules! join_tags {
    ($TargetModel:ident, w: $WorkspaceModel:ident::F.$workspace_field:ident, g: $GlobalModel:ident::F.$global_field:ident) => {{
        use rorm::field;
        use rorm::internal::field::Field;
        use rorm::Model;
        use $crate::models::$GlobalModel;
        use $crate::models::$TargetModel;
        use $crate::models::$WorkspaceModel;
        use $crate::models::GlobalTag;
        use $crate::models::WorkspaceTag;

        $crate::modules::filter::sqler::joins::JoinTags {
            table_alias: "tags",

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
    /// Get a join which retrieves a domain's tags in a postgres array.
    pub fn domain() -> Self {
        join_tags!(Domain, w: DomainWorkspaceTag::F.domain, g: DomainGlobalTag::F.domain)
    }

    /// Get a join which retrieves a host's tags in a postgres array.
    pub fn host() -> Self {
        join_tags!(Host, w: HostWorkspaceTag::F.host, g: HostGlobalTag::F.host)
    }

    /// Get a join which retrieves a port's tags in a postgres array.
    pub fn port() -> Self {
        join_tags!(Port, w: PortWorkspaceTag::F.port, g: PortGlobalTag::F.port)
    }

    /// Get a join which retrieves a service's tags in a postgres array.
    pub fn service() -> Self {
        join_tags!(Service, w: ServiceWorkspaceTag::F.service, g: ServiceGlobalTag::F.service)
    }

    /// Get a join which retrieves a http service's tags in a postgres array.
    pub fn http_service() -> Self {
        join_tags!(HttpService, w: HttpServiceWorkspaceTag::F.http_service, g: HttpServiceGlobalTag::F.http_service)
    }

    /// Change the alias used for the sub query
    pub fn alias(mut self, table_alias: &'static str) -> Self {
        self.table_alias = table_alias;
        self
    }
}

impl<'a> RawJoin<'a> for JoinTags {
    fn append(self, sql: &mut String, _values: &mut Vec<Value<'a>>) -> fmt::Result {
        let Self {
            table_alias,
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
            ) AS "{table_alias}" ON "{target}"."{target_uuid}" = "{table_alias}"."{target_uuid}""#,
        )
    }
}
