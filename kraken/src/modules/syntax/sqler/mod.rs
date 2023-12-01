mod value_sqler;

use std::fmt;
use std::fmt::Write;

use rorm::db::sql::value::Value;
use rorm::internal::field::Field;
use rorm::prelude::*;

use crate::models::{Port, PortGlobalTag, PortWorkspaceTag};
use crate::modules::syntax::sqler::value_sqler::{sql_ports, sql_tags, ValueSqler};
use crate::modules::syntax::{And, Not, Or, PortAST};

impl PortAST {
    /// Write additional joins required for the conditions to a string
    pub fn sql_join(&self, sql: &mut String) -> fmt::Result {
        const PORT: &str = Port::TABLE;
        const PORT_UUID: &str = <field!(Port::F.uuid)>::NAME;

        if self.tags.is_some() {
            write!(
                sql,
                r#" JOIN ({}) AS "tags" ON "{PORT}"."{PORT_UUID}" = "tags"."{PORT_UUID}""#,
                crate::tags_table!(Port, w: PortWorkspaceTag::F.port, g: PortGlobalTag::F.port)
            )?;
        }
        Ok(())
    }

    /// Write the conditions to a string
    pub fn sql_condition<'a>(
        &'a self,
        sql: &mut String,
        values: &mut Vec<Value<'a>>,
    ) -> fmt::Result {
        let len = sql.len();

        write!(sql, "true")?;
        if let Some(tags) = self.tags.as_ref() {
            write!(sql, " AND ")?;
            sql_or(tags, &mut *sql, &mut *values, sql_tags)?;
        }
        if let Some(ports) = self.ports.as_ref() {
            write!(sql, " AND ")?;
            sql_or(ports, &mut *sql, &mut *values, sql_ports)?;
        }

        if len == sql.len() {
            write!(sql, "true")?;
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

/// This macro expands to a [`format_args`] which produces a query
/// which resolves to something like `SELECT uuid, tags FROM table ...`
///
/// `table` and `uuid` will belong to a [`Model`] specified by the macro's first argument.
/// `tags` will be a postgres array of all tags (by name) associated with the `uuid`.
///
/// ## Example
/// ```ignore
/// tags_table!(Port, w: PortWorkspaceTag::F.port, g: PortGlobalTag::F.port)
/// ```
/// will resolve to
/// ```sql
/// SELECT
///     "port"."uuid",
///     array_agg("workspacetag"."name") || array_agg("globaltag"."name") AS "tags"
/// FROM
///     "port"
///         LEFT JOIN "portworkspacetag" ON "port"."uuid" = "portworkspacetag"."port"
///         LEFT JOIN "workspacetag" ON "portworkspacetag"."workspace_tag" = "workspacetag"."uuid"
///         LEFT JOIN "portglobaltag" ON "port"."uuid" = "portglobaltag"."port"
///         LEFT JOIN "globaltag" ON "portglobaltag"."global_tag" = "globaltag"."uuid"
/// GROUP BY
///     "port"."uuid"
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! tags_table {
    ($TargetModel:ident, w: $WorkspaceModel:ident::F.$workspace_field:ident, g: $GlobalModel:ident::F.$global_field:ident) => {{
        use crate::models::{WorkspaceTag, GlobalTag};

        const TARGET: &str = $TargetModel::TABLE;
        const TARGET_UUID: &str = <field!($TargetModel::F.uuid)>::NAME;

        const WORKSPACETAG: &str = WorkspaceTag::TABLE;
        const WORKSPACETAG_UUID: &str = <field!(WorkspaceTag::F.uuid)>::NAME;
        const WORKSPACETAG_NAME: &str = <field!(WorkspaceTag::F.name)>::NAME;

        const GLOBALTAG: &str = GlobalTag::TABLE;
        const GLOBALTAG_UUID: &str = <field!(GlobalTag::F.uuid)>::NAME;
        const GLOBALTAG_NAME: &str = <field!(GlobalTag::F.name)>::NAME;

        const M2M_WORKSPACETAG: &str = $WorkspaceModel::TABLE;
        const M2M_WORKSPACETAG_WORKSPACETAG: &str = <field!($WorkspaceModel::F.workspace_tag)>::NAME;
        const M2M_WORKSPACETAG_TARGET: &str = <field!($WorkspaceModel::F.$workspace_field)>::NAME;

        const M2M_GLOBALTAG: &str = $GlobalModel::TABLE;
        const M2M_GLOBALTAG_GLOBALTAG: &str = <field!($GlobalModel::F.global_tag)>::NAME;
        const M2M_GLOBALTAG_TARGET: &str = <field!($WorkspaceModel::F.$global_field)>::NAME;

        format_args!(r#"
            SELECT
                "{TARGET}"."{TARGET_UUID}",
                array_agg("{WORKSPACETAG}"."{WORKSPACETAG_NAME}") || array_agg("{GLOBALTAG}"."{GLOBALTAG_NAME}") AS "tags"
            FROM
                "{TARGET}"
                    LEFT JOIN "{M2M_WORKSPACETAG}" ON "{TARGET}"."{TARGET_UUID}" = "{M2M_WORKSPACETAG}"."{M2M_WORKSPACETAG_TARGET}"
                    LEFT JOIN "{WORKSPACETAG}" ON "{M2M_WORKSPACETAG}"."{M2M_WORKSPACETAG_WORKSPACETAG}" = "{WORKSPACETAG}"."{WORKSPACETAG_UUID}"
                    LEFT JOIN "{M2M_GLOBALTAG}" ON "{TARGET}"."{TARGET_UUID}" = "{M2M_GLOBALTAG}"."{M2M_GLOBALTAG_TARGET}"
                    LEFT JOIN "{GLOBALTAG}" ON "{M2M_GLOBALTAG}"."{M2M_GLOBALTAG_GLOBALTAG}" = "{GLOBALTAG}"."{GLOBALTAG_UUID}"
            GROUP BY
                "{TARGET}"."{TARGET_UUID}"
        "#)
    }};
}
