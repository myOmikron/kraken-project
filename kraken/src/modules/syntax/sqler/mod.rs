mod tags;
mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;

pub use self::tags::JoinTags;
use crate::modules::syntax::sqler::tags::sql_tags;
use crate::modules::syntax::sqler::value_sqler::{sql_ports, ValueSqler};
use crate::modules::syntax::{And, DomainAST, GlobalAST, HostAST, Not, Or, PortAST, ServiceAST};

impl GlobalAST {
    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        write!(sql, "true")?;

        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
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
        write!(sql, "true")?;

        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
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
        write!(sql, "true")?;

        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
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
        write!(sql, "true")?;

        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
        }
        if let Some(ports) = self.ports.as_ref() {
            write!(sql, " AND ")?;
            sql_or(ports, &mut *sql, &mut *values, sql_ports)?;
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
        write!(sql, "true")?;

        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
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
        [and] => sql_and(and, sql, values, sql_value),
        [first, rest @ ..] => {
            write!(sql, "(")?;
            sql_and(first, sql, values, sql_value)?;
            for and in rest {
                write!(sql, " OR ")?;
                sql_and(and, sql, values, sql_value)?;
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
    sql_value: impl ValueSqler<T>,
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
    sql_value: impl ValueSqler<T>,
) -> fmt::Result {
    if not.is_negated {
        write!(sql, "(NOT ")?;
        sql_value(&not.value, sql, values)?;
        write!(sql, ")")?;
        Ok(())
    } else {
        sql_value(&not.value, sql, values)
    }
}
