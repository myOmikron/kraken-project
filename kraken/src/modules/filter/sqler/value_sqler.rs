use std::fmt;
use std::fmt::Write;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::internal::relation_path::JoinAlias;
use rorm::prelude::*;

use super::super::{MaybeRange, Range};
use crate::models::PortProtocol;

/// Controls how values are written to sql
///
/// "Value" in this context means some comparison for a queried table to match the given value
pub trait ValueSqler<T> {
    /// Write a value to sql
    fn sql_value<'a>(
        &self,
        value: &'a T,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result;
}

/// Checks whether a tag is set on the aggregated model
///
/// Requires the [`JoinTags`] to be applied.
pub struct TagSqler;
impl<T: AsRef<str>> ValueSqler<T> for TagSqler {
    fn sql_value<'a>(
        &self,
        value: &'a T,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        values.push(Value::String(value.as_ref()));
        write!(
            sql,
            r#"(ARRAY[${i}]::VARCHAR[] <@ "tags"."tags")"#,
            i = values.len()
        )
    }
}

/// Checks the `created_at` column to lie in a certain range
pub struct CreatedAtSqler {
    table: &'static str,
    column: &'static str,
}
impl CreatedAtSqler {
    pub fn new<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
        }
    }
}
impl ValueSqler<Range<DateTime<Utc>>> for CreatedAtSqler {
    fn sql_value<'a>(
        &self,
        value: &'a Range<DateTime<Utc>>,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column } = *self;
        match value {
            Range {
                start: None,
                end: None,
            } => {
                write!(sql, "true")
            }
            Range {
                start: Some(start),
                end: None,
            } => {
                values.push(Value::ChronoDateTime(*start));
                write!(
                    sql,
                    r#"("{table}"."{column}" >= ${start})"#,
                    start = values.len()
                )
            }
            Range {
                start: None,
                end: Some(end),
            } => {
                values.push(Value::ChronoDateTime(*end));
                write!(
                    sql,
                    r#"("{table}"."{column}" <= ${end})"#,
                    end = values.len()
                )
            }
            Range {
                start: Some(start),
                end: Some(end),
            } => {
                values.push(Value::ChronoDateTime(*start));
                values.push(Value::ChronoDateTime(*end));
                write!(
                    sql,
                    r#"("{table}"."{column}" >= ${start} AND "{table}"."{column}" <= ${end})"#,
                    start = values.len() - 1,
                    end = values.len(),
                )
            }
        }
    }
}

/// Checks the `port` column to lie in a certain range or be a specific port
pub struct PortSqler {
    table: &'static str,
    column: &'static str,
}
impl PortSqler {
    pub fn new<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
        }
    }
}
impl ValueSqler<MaybeRange<u16>> for PortSqler {
    fn sql_value<'a>(
        &self,
        value: &'a MaybeRange<u16>,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column } = *self;
        match value {
            MaybeRange::Single(value) => {
                values.push(Value::I16(i16::from_ne_bytes(value.to_ne_bytes())));
                write!(sql, r#"("{table}"."{column}" = ${i})"#, i = values.len())
            }
            MaybeRange::Range(range) => match range {
                Range {
                    start: None,
                    end: None,
                } => {
                    write!(sql, "true")
                }
                Range {
                    start: Some(start),
                    end: None,
                } => {
                    values.push(Value::I16(i16::from_ne_bytes(start.to_ne_bytes())));
                    write!(
                        sql,
                        r#"("{table}"."{column}" >= ${start})"#,
                        start = values.len()
                    )
                }
                Range {
                    start: None,
                    end: Some(end),
                } => {
                    values.push(Value::I16(i16::from_ne_bytes(end.to_ne_bytes())));
                    write!(
                        sql,
                        r#"("{table}"."{column}" <= ${end})"#,
                        end = values.len()
                    )
                }
                Range {
                    start: Some(start),
                    end: Some(end),
                } => {
                    values.push(Value::I16(i16::from_ne_bytes(start.to_ne_bytes())));
                    values.push(Value::I16(i16::from_ne_bytes(end.to_ne_bytes())));
                    write!(
                        sql,
                        r#"("{table}"."{column}" >= ${start} AND "{table}"."{column}" <= ${end})"#,
                        start = values.len() - 1,
                        end = values.len(),
                    )
                }
            },
        }
    }
}

/// Like [`PortSqler`] but it handles `NULL` values:
///
/// A `NULL` never lies in a range and is not a specific port.
/// The difference to [`PortSqler`] is that
/// in sql `port = ?` and `NOT port = ?` both evaluate to `false`, if `port` is `NULL`.
pub struct NullablePortSqler(pub PortSqler);
impl ValueSqler<MaybeRange<u16>> for NullablePortSqler {
    fn sql_value<'a>(
        &self,
        value: &'a MaybeRange<u16>,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self(PortSqler { table, column }) = *self;
        write!(sql, r#"("{table}"."{column}" IS NOT NULL AND "#)?;
        self.0.sql_value(value, sql, values)?;
        write!(sql, ")")
    }
}

/// Checks the `ip_addr` column to lie in a certain ip network
pub struct IpSqler {
    table: &'static str,
    column: &'static str,
}
impl IpSqler {
    pub fn new<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
        }
    }
}
impl ValueSqler<IpNetwork> for IpSqler {
    fn sql_value<'a>(
        &self,
        value: &'a IpNetwork,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column } = *self;
        values.push(Value::IpNetwork(*value));
        write!(sql, r#"("{table}"."{column}" <<= ${i})"#, i = values.len())
    }
}

/// Checks a string column to be equal to a certain value
pub struct StringEqSqler {
    table: &'static str,
    column: &'static str,
}
impl StringEqSqler {
    pub fn new<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
        }
    }
}
impl<T: AsRef<str>> ValueSqler<T> for StringEqSqler {
    fn sql_value<'a>(
        &self,
        value: &'a T,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column } = *self;
        values.push(Value::String(value.as_ref()));
        write!(sql, r#"("{table}"."{column}" = ${i})"#, i = values.len())
    }
}

/// Checks a [`PortProtocol`] column to be equal to a certain value
pub struct PortProtocolSqler {
    table: &'static str,
    column: &'static str,
}
impl PortProtocolSqler {
    pub fn new<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
        }
    }
}
impl ValueSqler<PortProtocol> for PortProtocolSqler {
    fn sql_value<'a>(
        &self,
        value: &'a PortProtocol,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column } = *self;
        values.push(Value::Choice(match value {
            PortProtocol::Unknown => stringify!(Unknown),
            PortProtocol::Tcp => stringify!(Tcp),
            PortProtocol::Udp => stringify!(Udp),
            PortProtocol::Sctp => stringify!(Sctp),
        }));
        write!(sql, r#"("{table}"."{column}" = ${i})"#, i = values.len())
    }
}
