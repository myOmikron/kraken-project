use std::fmt;
use std::fmt::Write;
use std::marker::PhantomData;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use rorm::crud::selector::Selector;
use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::internal::relation_path::JoinAlias;
use rorm::prelude::*;

use super::super::MaybeRange;
use super::super::Range;
use crate::api::handler::services::schema::ServiceProtocols;
use crate::models::OsType;
use crate::models::PortProtocol;
use crate::modules::filter::ServiceTransport;
use crate::modules::raw_query::RawQueryBuilder;

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

/// A single column to compare against
pub struct Column<Cmp = ()> {
    pub column: &'static str,
    pub table: &'static str,
    pub phantom: PhantomData<Cmp>,
}
impl Column<()> {
    /// Construct a new column
    pub fn new(table: &'static str, column: &'static str) -> Self {
        Self {
            table,
            column,
            phantom: PhantomData,
        }
    }

    /// Construct `"tags"."tags"`
    pub fn tags() -> Self {
        Self::new("tags", "tags")
    }

    /// Construct a column known to rorm
    pub fn rorm<A: FieldAccess>(_: A) -> Self {
        Self {
            table: A::Path::ALIAS,
            column: A::Field::NAME,
            phantom: PhantomData,
        }
    }
}
impl<Cmp> Column<Cmp> {
    /// Compare the column to be equal to a specific value
    pub fn eq(&self) -> Column<CmpEq> {
        self.cmp()
    }

    /// Compare the nullable column to be equal to a specific value
    pub fn nullable_eq(&self) -> Column<CmpNullable<CmpEq>> {
        self.cmp()
    }

    /// Compare the column to lie in a specific range
    pub fn range(&self) -> Column<CmpRange> {
        self.cmp()
    }

    /// Compare the nullable column to lie in a specific range
    pub fn nullable_range(&self) -> Column<CmpNullable<CmpRange>> {
        self.cmp()
    }

    /// Compare the column to be equal to a specific value or to lie in a specific range
    pub fn maybe_range(&self) -> Column<CmpMaybeRange> {
        self.cmp()
    }

    /// Compare the nullable column to be equal to a specific value or to lie in a specific range
    pub fn nullable_maybe_range(&self) -> Column<CmpNullable<CmpMaybeRange>> {
        self.cmp()
    }

    /// Check the column (storing a postgres array) to contain a specific value
    pub fn contains(&self) -> Column<CmpContains> {
        self.cmp()
    }

    /// Check the column to be a subnet of (or equal to) a specific network
    pub fn subnet(&self) -> Column<CmpSubnet> {
        self.cmp()
    }

    /// Check the column to have all the input bits on a numerical value set.
    pub fn bitset(&self) -> Column<CmpBitset> {
        self.cmp()
    }

    fn cmp<NewCmp>(&self) -> Column<NewCmp> {
        Column {
            column: self.column,
            table: self.table,
            phantom: PhantomData,
        }
    }

    /// Check the column to match a subquery
    pub fn in_subquery<'a, S: Selector>(
        &self,
        selector: S,
        build: impl FnOnce(&mut RawQueryBuilder<'a, S>),
    ) -> impl FnOnce(&mut String, &mut Vec<Value<'a>>) -> fmt::Result {
        let Self { column, table, .. } = *self;
        move |sql, values| {
            write!(sql, r#""{table}"."{column}" IN "#)?;
            RawQueryBuilder::write_subquery(sql, values, selector, build);
            Ok(())
        }
    }
}

pub struct CmpEq;
impl<T: AsValue> ValueSqler<T> for Column<CmpEq> {
    fn sql_value<'a>(
        &self,
        value: &'a T,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
        match value.as_value() {
            Value::Choice(value) => {
                write!(sql, r#"("{table}"."{column}" = '{value}')"#)
            }
            value => {
                values.push(value);
                write!(sql, r#"("{table}"."{column}" = ${i})"#, i = values.len())
            }
        }
    }
}

pub struct CmpRange;
impl<T: AsValue> ValueSqler<Range<T>> for Column<CmpRange> {
    fn sql_value<'a>(
        &self,
        value: &'a Range<T>,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
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
                values.push(start.as_value());
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
                values.push(end.as_value());
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
                values.push(start.as_value());
                values.push(end.as_value());
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

pub struct CmpMaybeRange;
impl<T: AsValue> ValueSqler<MaybeRange<T>> for Column<CmpMaybeRange> {
    fn sql_value<'a>(
        &self,
        value: &'a MaybeRange<T>,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
        match value {
            MaybeRange::Single(value) => {
                values.push(value.as_value());
                write!(sql, r#"("{table}"."{column}" = ${i})"#, i = values.len())
            }
            MaybeRange::Range(range) => self.range().sql_value(range, sql, values),
        }
    }
}

pub struct CmpNullable<Cmp>(Cmp);
impl<Cmp, V> ValueSqler<V> for Column<CmpNullable<Cmp>>
where
    Column<Cmp>: ValueSqler<V>,
{
    fn sql_value<'a>(
        &self,
        value: &'a V,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
        write!(sql, r#"("{table}"."{column}" IS NOT NULL AND "#)?;
        self.cmp::<Cmp>().sql_value(value, sql, values)?;
        write!(sql, ")")
    }
}

pub struct CmpContains;
impl<T: AsValue + HasDbType> ValueSqler<T> for Column<CmpContains> {
    fn sql_value<'a>(
        &self,
        value: &'a T,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = self;
        values.push(value.as_value());
        write!(
            sql,
            r#"(ARRAY[${i}]::{db_type}[] <@ "{table}"."{column}")"#,
            i = values.len(),
            db_type = T::db_type()
        )
    }
}

pub struct CmpSubnet;
impl ValueSqler<IpNetwork> for Column<CmpSubnet> {
    fn sql_value<'a>(
        &self,
        value: &'a IpNetwork,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
        values.push(value.as_value());
        write!(sql, r#"("{table}"."{column}" <<= ${i})"#, i = values.len())
    }
}

pub struct CmpBitset;
impl<V: AsValue> ValueSqler<V> for Column<CmpBitset> {
    fn sql_value<'a>(
        &self,
        value: &'a V,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let Self { table, column, .. } = *self;
        values.push(value.as_value());
        write!(
            sql,
            r#"("{table}"."{column}" & ${i} = ${i})"#,
            i = values.len()
        )
    }
}

/// Small helper trait which converts `&T` into a [`Value`]
///
/// Unlike `rorm`'s [`FieldType`](rorm::fields::traits::FieldType),
/// this trait produces the [`Value`] type from `rorm-sql` and always just one of it.
pub trait AsValue {
    /// Convert `&self` into a [`Value`]
    fn as_value(&self) -> Value;
}
impl AsValue for str {
    fn as_value(&self) -> Value {
        Value::String(self)
    }
}
impl AsValue for String {
    fn as_value(&self) -> Value {
        Value::String(self)
    }
}
impl AsValue for PortProtocol {
    fn as_value(&self) -> Value {
        Value::Choice(match self {
            PortProtocol::Unknown => stringify!(Unknown),
            PortProtocol::Tcp => stringify!(Tcp),
            PortProtocol::Udp => stringify!(Udp),
            PortProtocol::Sctp => stringify!(Sctp),
        })
    }
}
impl AsValue for OsType {
    fn as_value(&self) -> Value {
        Value::Choice(match self {
            OsType::Unknown => stringify!(Unknown),
            OsType::Linux => stringify!(Linux),
            OsType::Windows => stringify!(Windows),
            OsType::Apple => stringify!(Apple),
            OsType::Android => stringify!(Android),
            OsType::FreeBSD => stringify!(FreeBSD),
        })
    }
}
impl AsValue for ServiceTransport {
    fn as_value(&self) -> Value {
        match self {
            ServiceTransport::Raw => Value::I16(ServiceProtocols::bitset_raw()),
            ServiceTransport::Tls => Value::I16(ServiceProtocols::bitset_tls()),
        }
    }
}
impl AsValue for bool {
    fn as_value(&self) -> Value {
        Value::Bool(*self)
    }
}
impl AsValue for u16 {
    fn as_value(&self) -> Value {
        Value::I32(*self as i32)
    }
}
impl AsValue for DateTime<Utc> {
    fn as_value(&self) -> Value {
        Value::ChronoDateTime(*self)
    }
}
impl AsValue for IpNetwork {
    fn as_value(&self) -> Value {
        Value::IpNetwork(*self)
    }
}

/// Small helper trait which associates a postgres type with a rust type
pub trait HasDbType {
    /// Get the associated postgres type
    fn db_type() -> &'static str;
}
impl HasDbType for str {
    fn db_type() -> &'static str {
        "VARCHAR"
    }
}
impl HasDbType for String {
    fn db_type() -> &'static str {
        "VARCHAR"
    }
}
