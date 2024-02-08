mod joins;
mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::crud::selector::Selector;
use rorm::db::sql::value::Value;
use rorm::prelude::*;

use crate::models::{Domain, Host, Port, Service};
use crate::modules::filter::sqler::joins::{JoinPorts, JoinTags};
use crate::modules::filter::sqler::value_sqler::{Column, ValueSqler};
use crate::modules::filter::{And, DomainAST, GlobalAST, HostAST, Not, Or, PortAST, ServiceAST};
use crate::modules::raw_query::RawQueryBuilder;

impl DomainAST {
    /// Apply the domain specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
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
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Domain::F.created_at).range());
        add_ast_field(sql, domains, Column::rorm(Domain::F.domain).eq());

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Domain::F.created_at).range());
    }
}
impl HostAST {
    /// Apply the host specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
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
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Host::F.created_at).range());
        add_ast_field(sql, ips, Column::rorm(Host::F.ip_addr).subnet());

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Host::F.created_at).range());
    }
}
impl PortAST {
    /// Apply the port specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
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
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Port::F.created_at).range());
        add_ast_field(sql, ports, Column::rorm(Port::F.port).maybe_range());
        add_ast_field(sql, ips, Column::rorm(Port::F.host.ip_addr).subnet());
        add_ast_field(sql, protocols, Column::rorm(Port::F.protocol).eq());

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Port::F.created_at).range());
    }
}
impl ServiceAST {
    /// Apply the service specific ast as well as the global ast to a query builder.
    ///
    /// The query builder has to be in its `join` position and might end in its `where` position.
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
            services,
            ports,
        } = self;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Service::F.created_at).range());
        add_ast_field(sql, ips, Column::rorm(Service::F.host.ip_addr).subnet());
        add_ast_field(sql, services, Column::rorm(Service::F.name).eq());
        add_ast_field(
            sql,
            ports,
            Column::rorm(Port::F.port).nullable_maybe_range(), // This table is joined manually
        );

        let GlobalAST { tags, created_at } = global;
        add_ast_field(sql, tags, Column::tags().contains());
        add_ast_field(sql, created_at, Column::rorm(Service::F.created_at).range());
    }
}

/// Helper function to be called from `...AST::apply_to_query`
///
/// It checks a field to be `Some` and appends it to the query builder
pub fn add_ast_field<'a, T>(
    query_builder: &mut RawQueryBuilder<'a, impl Selector>,
    field: &'a Option<Or<T>>,
    sql_value: impl ValueSqler<T>,
) {
    if let Some(field) = field {
        query_builder.append_condition(|sql, values| sql_or(field, sql, values, sql_value))
    }
}

/// Write an [`Or`] expression to sql using a [`ValueSqler`] to write the leaves
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

/// Write an [`And`] expression to sql using a [`ValueSqler`] to write the leaves
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

/// Write a [`Not`] to sql expression using a [`ValueSqler`] to write the potentially negated value
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
