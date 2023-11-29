use std::fmt::Debug;

/// Port AST
#[derive(Default, Debug)]
pub struct PortAST {
    pub tags: Option<Or<String>>,
    pub ports: Option<Or<MaybeRange<u16>>>,
}

/// OR expression
#[derive(Debug, Clone)]
pub struct Or<T>(pub Vec<And<T>>);

/// AND expression
#[derive(Debug, Clone)]
pub struct And<T>(pub Vec<Not<T>>);

/// Potentially negated expression
#[derive(Debug, Clone)]
pub struct Not<T> {
    pub is_negated: bool,
    pub value: T,
}

/// Range of values or a single one
#[derive(Debug, Clone)]
pub enum MaybeRange<T> {
    /// Range of values
    Range(Range<T>),
    /// Single value
    Single(T),
}

/// A range of values
#[derive(Debug, Clone)]
pub struct Range<T> {
    /// Start of the range
    pub start: Option<T>,
    /// End of the range
    pub end: Option<T>,
}
