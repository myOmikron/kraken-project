mod joins;
mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;
use rorm::prelude::*;

pub use self::joins::*;
use crate::models::{Domain, Host, Port, Service};
use crate::modules::syntax::sqler::value_sqler::{
    IpSqler, NullablePortSqler, PortProtocolSqler, PortSqler, StringEqSqler, TagSqler, ValueSqler,
};
use crate::modules::syntax::{And, DomainAST, GlobalAST, HostAST, Not, Or, PortAST, ServiceAST};

impl GlobalAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { tags } = self;

        write!(sql, "true")?;

        if let Some(tags) = tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, TagSqler)?;
        }

        Ok(())
    }
}
impl DomainAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { tags, domains } = self;

        write!(sql, "true")?;

        if let Some(tags) = tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, TagSqler)?;
        }
        if let Some(domains) = domains.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                domains,
                &mut *sql,
                &mut *values,
                StringEqSqler::new(Domain::F.domain),
            )?;
        }

        Ok(())
    }
}
impl HostAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { tags, ips } = self;

        write!(sql, "true")?;

        if let Some(tags) = tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, TagSqler)?;
        }
        if let Some(ips) = ips.as_ref() {
            write!(sql, " AND ")?;
            sql_or(ips, &mut *sql, &mut *values, IpSqler::new(Host::F.ip_addr))?;
        }

        Ok(())
    }
}
impl PortAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self {
            tags,
            ports,
            ips,
            protocols,
        } = self;

        write!(sql, "true")?;

        if let Some(tags) = tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, TagSqler)?;
        }
        if let Some(ports) = ports.as_ref() {
            write!(sql, " AND ")?;
            sql_or(ports, &mut *sql, &mut *values, PortSqler::new(Port::F.port))?;
        }
        if let Some(ips) = ips.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                ips,
                &mut *sql,
                &mut *values,
                IpSqler::new(Port::F.host.ip_addr),
            )?;
        }
        if let Some(protocols) = protocols.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                protocols,
                &mut *sql,
                &mut *values,
                PortProtocolSqler::new(Port::F.protocol),
            )?;
        }

        Ok(())
    }
}
impl ServiceAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self {
            tags,
            ips,
            names,
            ports,
        } = self;

        write!(sql, "true")?;

        if let Some(tags) = tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, TagSqler)?;
        }
        if let Some(ips) = ips.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                ips,
                &mut *sql,
                &mut *values,
                IpSqler::new(Service::F.host.ip_addr),
            )?;
        }
        if let Some(names) = names.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                names,
                &mut *sql,
                &mut *values,
                StringEqSqler::new(Service::F.name),
            )?;
        }
        if let Some(ports) = ports.as_ref() {
            write!(sql, " AND ")?;
            sql_or(
                ports,
                &mut *sql,
                &mut *values,
                NullablePortSqler(PortSqler::new(Port::F.port)), // This table is joined manually
            )?;
        }

        Ok(())
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
