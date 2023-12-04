mod joins;
mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::crud::selector::Selector;
use rorm::db::sql::value::Value;
use rorm::prelude::*;

use crate::models::{Domain, Host, Port, Service};
use crate::modules::raw_query::RawQueryBuilder;
use crate::modules::syntax::sqler::joins::{JoinPorts, JoinTags};
use crate::modules::syntax::sqler::value_sqler::{
    CreatedAtSqler, IpSqler, NullablePortSqler, PortProtocolSqler, PortSqler, StringEqSqler,
    TagSqler, ValueSqler,
};
use crate::modules::syntax::{And, DomainAST, GlobalAST, HostAST, Not, Or, PortAST, ServiceAST};

impl DomainAST {
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::domain());
        }

        let DomainAST {
            tags,
            created_at,
            domains,
        } = self;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Domain::F.created_at));
        add_ast_field(sql, domains, StringEqSqler::new(Domain::F.domain));

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Domain::F.created_at));
    }
}
impl HostAST {
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::host());
        }

        let HostAST {
            tags,
            created_at,
            ips,
        } = self;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Host::F.created_at));
        add_ast_field(sql, ips, IpSqler::new(Host::F.ip_addr));

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Host::F.created_at));
    }
}
impl PortAST {
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::port());
        }

        let PortAST {
            tags,
            created_at,
            ports,
            ips,
            protocols,
        } = self;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Port::F.created_at));
        add_ast_field(sql, ports, PortSqler::new(Port::F.port));
        add_ast_field(sql, ips, IpSqler::new(Port::F.host.ip_addr));
        add_ast_field(sql, protocols, PortProtocolSqler::new(Port::F.protocol));

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Port::F.created_at));
    }
}
impl ServiceAST {
    pub fn apply_to_query<'a>(
        &'a self,
        global: &'a GlobalAST,
        sql: &mut RawQueryBuilder<'a, impl Selector>,
    ) {
        if self.tags.is_some() || global.tags.is_some() {
            sql.append_join(JoinTags::service());
        }
        if self.ports.is_some() {
            sql.append_join(JoinPorts);
        }

        let ServiceAST {
            tags,
            created_at,
            ips,
            names,
            ports,
        } = self;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Service::F.created_at));
        add_ast_field(sql, ips, IpSqler::new(Service::F.host.ip_addr));
        add_ast_field(sql, names, StringEqSqler::new(Service::F.name));
        add_ast_field(
            sql,
            ports,
            NullablePortSqler(PortSqler::new(Port::F.port)), // This table is joined manually
        );

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, TagSqler);
        add_ast_field(sql, created_at, CreatedAtSqler::new(Service::F.created_at));
    }
}

pub fn add_ast_field<'a, T>(
    query_builder: &mut RawQueryBuilder<'a, impl Selector>,
    field: &'a Option<Or<T>>,
    sql_value: impl ValueSqler<T>,
) {
    if let Some(field) = field {
        query_builder.append_condition(|sql, values| sql_or(field, sql, values, sql_value))
    }
}

pub fn sql_or<'a, T>(
    or: &'a Or<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: impl ValueSqler<T>,
) -> fmt::Result {
    match or.0.as_slice() {
        [] => write!(sql, "true"),
        [and] => sql_and(and, sql, values, &sql_value),
        [first, rest @ ..] => {
            write!(sql, "(")?;
            sql_and(first, sql, values, &sql_value)?;
            for and in rest {
                write!(sql, " OR ")?;
                sql_and(and, sql, values, &sql_value)?;
            }
            write!(sql, ")")?;
            Ok(())
        }
    }
}

pub fn sql_and<'a, T>(
    and: &'a And<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: &impl ValueSqler<T>,
) -> fmt::Result {
    match and.0.as_slice() {
        [] => write!(sql, "false"),
        [not] => sql_not(not, sql, values, sql_value),
        [first, rest @ ..] => {
            write!(sql, "(")?;
            sql_not(first, sql, values, sql_value)?;
            for not in rest {
                write!(sql, " AND ")?;
                sql_not(not, sql, values, sql_value)?;
            }
            write!(sql, ")")?;
            Ok(())
        }
    }
}

pub fn sql_not<'a, T>(
    not: &'a Not<T>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
    sql_value: &impl ValueSqler<T>,
) -> fmt::Result {
    if not.is_negated {
        write!(sql, "(NOT ")?;
        sql_value.sql_value(&not.value, sql, values)?;
        write!(sql, ")")?;
        Ok(())
    } else {
        sql_value.sql_value(&not.value, sql, values)
    }
}
