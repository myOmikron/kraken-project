//! this module hold the code for the abstract syntax tree

use std::fmt::Debug;

use chrono::{DateTime, Utc};

use crate::modules::syntax::Parser;

/// OR expression
#[derive(Debug, Clone)]
pub struct Or<T>(pub Vec<And<T>>);

/// AND expression
#[derive(Debug, Clone)]
pub struct And<T>(pub Vec<Not<T>>);

/// Negate expression
#[derive(Debug, Clone)]
pub struct Not<T>(pub bool, pub T);

/// ranged expression
#[derive(Debug, Clone)]
pub enum MaybeRange<T> {
    /// ranged
    Range {
        /// start of the range
        start: Option<T>,
        /// end of the range
        end: Option<T>,
    },
    /// single value
    Single(T),
}

/// Global AST
#[derive(Default, Debug)]
pub struct GlobalAST {
    tags: Option<Or<String>>,
    created_at: Option<Or<MaybeRange<DateTime<Utc>>>>,
}

impl GlobalAST {
    /// Parse Global AST
    pub fn parse(parser: &Parser<'_>) -> Result<GlobalAST, String> {
        let mut ast = GlobalAST::default();

        loop {
            let Some(field) = parser.parse_field().map_err(|e| format!("{e}"))? else {
                return Err(String::from("no fields"));
            };

            match field.as_str() {
                "tags" => {
                    ast.tags = parse_expr(false, parser)?;
                }
                "created_at" => {
                    ast.created_at = parse_expr(true, parser)?;
                }
                _ => break, // done!
            }
        }

        Ok(ast)
    }
}

/// parse an expression
///
///
pub fn parse_expr<T: Clone + 'static>(
    ranged: bool,
    parser: &Parser<'_>,
) -> Result<Option<T>, String> {
    Ok(parser
        .parse_statement(ranged)
        .map_err(|e| format!("parser error: {e}"))?
        .and_then(|v| v.downcast_ref::<T>().cloned()))
}
