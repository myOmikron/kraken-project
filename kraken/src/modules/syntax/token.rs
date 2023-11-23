/// Token classification
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
    /// a string like token
    Word,
    /// number: only `i64` for now
    Number,
    /// uuid format `ba6eb330-4f7f-11eb-a2fb-67c34e9ac07c`
    Uuid,
    /// Date format `"YYYY-MM-DD"`
    Date,
    /// DateTime format `"YYYY-MM-DD HH-MM"`
    DateTime,

    // --- Operators
    /// represented as a `,`
    LogicalOr,
    /// represented as a `&`
    LogicalAnd,
    /// represented as a `:`
    LogicalEqual,
    /// represented as a `!`
    LogicalNegate,
    /// represented as a `-`
    RangeOperator,

    // --- Special
    /// represented as a space
    WhiteSpace,
    /// any token other than the defined ones
    Bad,
    /// reached the end of input
    Eof,
}

/// A token is the [`TokenKind`] and a reference to the
/// part in the input
#[derive(Debug, Copy, Clone)]
pub struct Token<'src> {
    pub(crate) kind: TokenKind,
    pub(crate) literal: &'src str,
}

impl<'src> Token<'src> {
    /// constructs a new [`Token`]
    pub(crate) fn new(kind: TokenKind, literal: &'src str) -> Self {
        Self { kind, literal }
    }
}
