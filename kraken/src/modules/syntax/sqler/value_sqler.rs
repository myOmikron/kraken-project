use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::prelude::*;

use super::super::{MaybeRange, Range};
use crate::models::Port;

/// Trait alias for `for<'a> Fn(&'a T, &mut String, &mut Vec<Value<'a>>) -> fmt::Result` and `Copy`
pub trait ValueSqler<T>:
    for<'a> Fn(&'a T, &mut String, &mut Vec<Value<'a>>) -> fmt::Result + Copy
{
}
impl<T, F: for<'a> Fn(&'a T, &mut String, &mut Vec<Value<'a>>) -> fmt::Result + Copy> ValueSqler<T>
    for F
{
}

pub fn sql_tags<'a>(tag: &'a String, sql: &mut String, values: &mut Vec<Value<'a>>) -> fmt::Result {
    values.push(Value::String(tag));
    write!(
        sql,
        r#"(ARRAY[${i}]::VARCHAR[] <@ "tags"."tags")"#,
        i = values.len()
    )
}

pub fn sql_ports<'a>(
    port: &'a MaybeRange<u16>,
    sql: &mut String,
    values: &mut Vec<Value<'a>>,
) -> fmt::Result {
    const TABLE: &str = Port::TABLE;
    const COLUMN: &str = <field!(Port::F.port) as Field>::NAME;

    match port {
        MaybeRange::Single(value) => {
            values.push(Value::I16(i16::from_ne_bytes(value.to_ne_bytes())));
            write!(sql, r#"("{TABLE}"."{COLUMN}" = ${i})"#, i = values.len())
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
                    r#"("{TABLE}"."{COLUMN}" >= ${start})"#,
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
                    r#"("{TABLE}"."{COLUMN}" <= ${end})"#,
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
                    r#"("{TABLE}"."{COLUMN}" >= ${start} AND "{TABLE}"."{COLUMN}" <= ${end})"#,
                    start = values.len() - 1,
                    end = values.len(),
                )
            }
        },
    }
}
