//! A [`RawQueryBuilder`] for constructing raw sql queries when [`rorm`] is not good enough

use std::fmt;
use std::fmt::Write;
use std::mem::swap;

use futures::Stream;
use futures::StreamExt;
use log::debug;
use rorm::crud::decoder::Decoder;
use rorm::crud::selector::Selector;
use rorm::db::database::ColumnSelector;
use rorm::db::database::JoinTable;
use rorm::db::executor;
use rorm::db::sql::aggregation::SelectAggregator;
use rorm::db::sql::conditional::BuildCondition;
use rorm::db::sql::value::Value;
use rorm::db::sql::DBImpl;
use rorm::db::Executor;
use rorm::internal::field::Field;
use rorm::internal::field::FieldProxy;
use rorm::internal::field::SingleColumnField;
use rorm::internal::query_context::QueryContext;
use rorm::internal::relation_path::Path;
use rorm::Model;

fn handle_fmt(block: impl FnOnce() -> fmt::Result) {
    // This function functions as the single consumer of fmt::Result which is produce by writing raw sql to a string
    // If we can't write to a string, we can just abort everything
    #[allow(clippy::expect_used)]
    block().expect("Formatting a string shouldn't fail");
}

/// Helper struct for constructing raw sql queries without abandoning all handrails.
///
/// This struct is basically a wrapper around a `&mut String` to write the sql to
/// and a `&mut Vec<Value>>` to write values to bind to.
///
/// It is constructed with [`RawQueryBuilder::new`] which takes a `rorm` [`Selector`]
/// (i.e. the thing in [`query!`](rorm::query)) to construct the initial `SELECT` and `FROM` clauses.
/// Then it provides several helper methods to append raw sql snippets in somewhat controlled manners.
/// Finally it provides [`RawQueryBuilder::one`] or [`RawQueryBuilder::stream`] to execute the query.
///
/// ## Beware
/// This is a crude helper, not `rorm`'s `QueryBuilder` which helps you in the type system!
/// All methods are naive "append" operations with a rudimentary check,
/// whether you're appending snippets in the wrong order.
///
/// ## Closure arguments
/// Most methods will take a closure as argument which performs the appending
/// instead of just exposing the underlying string and vector.
///
/// This inversion of control serves a few purposes:
/// - The builder can perform some rudimentary sanity-checks before performing the operation.
/// - The caller can use `write!` and simply propagate the `fmt::Result` with `?`.
/// - The transition from one section to another is performed by the builder lazily.
pub struct RawQueryBuilder<'a, S: Selector> {
    decoder: S::Decoder,
    sql: String,
    values: Vec<Value<'a>>,
    position: QueryBuilderPosition,
}
impl<'a, S: Selector> RawQueryBuilder<'a, S> {
    /// Construct a new builder.
    ///
    /// The `SELECT` clause will be constructed to match the `selector`.
    ///
    /// The `FROM` clause will be initialised based on `selector`
    /// but can be appended to via [`RawQueryBuilder::append_join`].
    pub fn new(selector: S) -> Self {
        let mut ctx = QueryContext::new();
        let decoder = selector.select(&mut ctx);
        let mut sql = String::new();

        handle_fmt(|| {
            build_select(&ctx.get_selects(), &mut sql)?;
            write!(sql, " FROM \"{}\"", S::Model::TABLE)?;
            for join in ctx.get_joins() {
                build_join(&join, &mut sql)?;
            }
            Ok(())
        });

        Self {
            decoder,
            sql,
            values: Vec::new(),
            position: QueryBuilderPosition::Join,
        }
    }

    /// Use the handles (`sql` and `values`) which are passed out by methods like [`append_condition`]
    /// to write a subquery.
    ///
    /// The subquery will be automatically wrapped in parentheses.
    pub fn write_subquery(
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
        selector: S,
        write: impl FnOnce(&mut RawQueryBuilder<'a, S>),
    ) {
        let mut builder = RawQueryBuilder::new(selector);
        sql.push('(');
        sql.push_str(&builder.sql); // Write the "select from" generated in RawQueryBuilder::new
        swap(sql, &mut builder.sql);
        swap(values, &mut builder.values);
        write(&mut builder);
        swap(sql, &mut builder.sql);
        swap(values, &mut builder.values);
        sql.push(')');
    }

    /// Append a `JOIN`
    ///
    /// ```ignore
    /// query.append_join(|sql, _| write!(sql, " JOIN table ON some_condition"));
    /// ```
    pub fn append_join(&mut self, join: impl RawJoin<'a>) {
        assert!(self.position <= QueryBuilderPosition::Join);

        handle_fmt(move || join.append(&mut self.sql, &mut self.values));
    }

    /// Append a condition to the `WHERE` clause.
    ///
    /// The conditions from multiple invocations will be `AND`ed implicitly.
    ///
    /// ```ignore
    /// query.append_condition(|sql, values| {
    ///     values.push(Value::String("bar"));
    ///     write!(sql, "foo = ${}", values.len())
    /// });
    /// ```
    pub fn append_condition(
        &mut self,
        append: impl FnOnce(&mut String, &mut Vec<Value<'a>>) -> fmt::Result,
    ) {
        assert!(self.position <= QueryBuilderPosition::Where);

        handle_fmt(|| {
            // join multiple conditions with an implicit AND
            if self.position < QueryBuilderPosition::Where {
                self.position = QueryBuilderPosition::Where;
                write!(&mut self.sql, " WHERE ")?;
            } else {
                write!(&mut self.sql, " AND ")?;
            }

            append(&mut self.sql, &mut self.values)
        });
    }

    /// Append a `"table"."column" = ${}` to the `WHERE` clause
    ///
    /// This method is a specialization of [`RawQueryBuilder::append_condition`]
    /// which provides some convenience by taking a `rorm` field.
    /// It does not handle implicit joins. Therefore, only direct fields are allowed.
    ///
    /// ```ignore
    /// query.append_eq_condition(
    ///     Port::F.uuid,
    ///     Value::Uuid(Uuid::new_v4()),
    /// );
    /// ```
    pub fn append_eq_condition<F: Field>(
        &mut self,
        _field: FieldProxy<F, S::Model>,
        value: Value<'a>,
    ) {
        self.append_condition(move |sql, values| {
            values.push(value);
            write!(
                sql,
                "\"{table}\".\"{column}\" = ${i}",
                table = S::Model::TABLE,
                column = F::NAME,
                i = values.len()
            )
        })
    }

    /// Order the query by a field in ascending order.
    ///
    /// This method does not handle implicit joins. Therefore, only direct fields are allowed.
    pub fn order_asc<F: SingleColumnField>(&mut self, field: FieldProxy<F, S::Model>) {
        self.order(field, true)
    }

    /// Order the query by a field in descending order.
    ///
    /// It does not handle implicit joins. Therefore, only direct fields are allowed.
    pub fn order_desc<F: SingleColumnField>(&mut self, field: FieldProxy<F, S::Model>) {
        self.order(field, false)
    }

    fn order<F: Field, P: Path>(&mut self, _field: FieldProxy<F, P>, asc: bool) {
        assert!(self.position <= QueryBuilderPosition::Order);

        handle_fmt(|| {
            if self.position < QueryBuilderPosition::Order {
                write!(&mut self.sql, " ORDER BY ")?;
            } else {
                write!(&mut self.sql, ", ")?;
            }

            write!(
                &mut self.sql,
                "\"{table}\".\"{column}\" {direction}",
                table = P::ALIAS,
                column = F::NAME,
                direction = if asc { "ASC" } else { "DESC" },
            )
        });
    }

    /// Add a limit and an offset to the query
    pub fn limit_offset(&mut self, limit: u64, offset: u64) {
        assert!(self.position < QueryBuilderPosition::LimOff);

        handle_fmt(|| {
            self.position = QueryBuilderPosition::LimOff;
            write!(&mut self.sql, " LIMIT {limit} OFFSET {offset}")
        });
    }

    /// Execute the query and retrieve exactly one result.
    ///
    /// This method does not set the queries limit. Please call [`RawQueryBuilder::limit_offset`] yourself.
    pub async fn one(mut self, executor: impl Executor<'_>) -> Result<S::Result, rorm::Error> {
        self.sql.push(';');
        debug!("Raw SQL: {}", self.sql);
        let row = executor
            .execute::<executor::One>(self.sql, self.values)
            .await?;
        self.decoder.by_name(&row)
    }

    /// Execute the query and retrieve a stream of result.
    pub fn stream<'e, 'r>(
        mut self,
        executor: impl Executor<'e>,
    ) -> impl Stream<Item = Result<S::Result, rorm::Error>> + 'r
    where
        'e: 'r,
        'a: 'r,
        S::Decoder: 'r,
    {
        self.sql.push(';');
        debug!("Raw SQL: {}", self.sql);
        executor
            .execute::<executor::Stream>(self.sql, self.values)
            .map(move |result| self.decoder.by_name(&result?))
    }
}

/// Some type which represents a raw join
pub trait RawJoin<'a> {
    /// Append the sql for the join
    fn append(self, sql: &mut String, values: &mut Vec<Value<'a>>) -> fmt::Result;
}
impl<'a, F: FnOnce(&mut String, &mut Vec<Value<'a>>) -> fmt::Result> RawJoin<'a> for F {
    fn append(self, sql: &mut String, values: &mut Vec<Value<'a>>) -> fmt::Result {
        self(sql, values)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum QueryBuilderPosition {
    Join,
    Where,
    Order,
    LimOff,
}

fn build_select(columns: &[ColumnSelector<'_>], sql: &mut String) -> fmt::Result {
    let mut columns = columns.iter();
    write!(sql, "SELECT ")?;
    match columns.next() {
        None => write!(sql, "*")?,
        Some(column) => {
            build_select_column(column, sql)?;
            for column in columns {
                write!(sql, ", ")?;
                build_select_column(column, sql)?;
            }
        }
    }
    Ok(())
}
fn build_select_column(column: &ColumnSelector<'_>, sql: &mut String) -> fmt::Result {
    let ColumnSelector {
        table_name,
        column_name,
        select_alias,
        aggregation,
    } = column;

    if let Some(aggregation) = aggregation {
        match aggregation {
            SelectAggregator::Avg => write!(sql, "AVG(")?,
            SelectAggregator::Count => write!(sql, "COUNT(")?,
            SelectAggregator::Sum => write!(sql, "SUM(")?,
            SelectAggregator::Max => write!(sql, "MAX(")?,
            SelectAggregator::Min => write!(sql, "MIN(")?,
        }
    }

    if let Some(table_name) = table_name {
        write!(sql, "\"{table_name}\".")?;
    }
    write!(sql, "\"{column_name}\"")?;

    if aggregation.is_some() {
        write!(sql, ")")?;
    }

    if let Some(alias) = select_alias {
        write!(sql, " AS \"{alias}\"")?;
    }

    Ok(())
}
fn build_join(join: &JoinTable, sql: &mut String) -> fmt::Result {
    let JoinTable {
        join_type,
        table_name,
        join_alias,
        join_condition,
    } = join;
    let mut values = Vec::new();
    write!(
        sql,
        " {join_type} \"{table_name}\" AS \"{join_alias}\" ON {}",
        join_condition.build(DBImpl::Postgres, &mut values)
    )?;
    assert_eq!(values.len(), 0);
    Ok(())
}
