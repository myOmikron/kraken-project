//! This module holds all the code for lexing & parsing the filter syntax

mod lexer;
mod parser;
mod sqler;

use std::error::Error as StdError;
use std::fmt::Debug;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use thiserror::Error;

use crate::models::PortProtocol;
use crate::modules::filter::lexer::{Token, UnexpectedCharacter};

/// AST for global filter
#[derive(Default, Debug)]
pub struct GlobalAST {
    /// Filter by tags
    pub tags: Option<Or<String>>,

    /// Filter by creation time
    pub created_at: Option<Or<Range<DateTime<Utc>>>>,
}

/// AST for domain specific filter
#[derive(Default, Debug)]
pub struct DomainAST {
    /// Filter by tags
    pub tags: Option<Or<String>>,

    /// Filter by creation time
    pub created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter by domain name
    pub domains: Option<Or<String>>,
}

/// AST for host specific filter
#[derive(Default, Debug)]
pub struct HostAST {
    /// Filter by tags
    pub tags: Option<Or<String>>,

    /// Filter by creation time
    pub created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter by ip address
    pub ips: Option<Or<IpNetwork>>,
}

/// AST for port specific filter
#[derive(Default, Debug)]
pub struct PortAST {
    /// Filter ports by tags
    pub tags: Option<Or<String>>,

    /// Filter by creation time
    pub created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter ports by port numbers
    pub ports: Option<Or<MaybeRange<u16>>>,

    /// Filter by ip address
    pub ips: Option<Or<IpNetwork>>,

    /// Filter by protocols
    pub protocols: Option<Or<PortProtocol>>,
}

/// AST for service specific filter
#[derive(Default, Debug)]
pub struct ServiceAST {
    /// Filter by tags
    pub tags: Option<Or<String>>,

    /// Filter by creation time
    pub created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter by ip address
    pub ips: Option<Or<IpNetwork>>,

    /// Filter by ports
    pub ports: Option<Or<MaybeRange<u16>>>,

    /// Filter by service name
    pub names: Option<Or<String>>,
}

/// An error encountered while parsing a filter ast
#[derive(Debug, Error)]
pub enum ParseError {
    /// The lexer encountered an unexpected character
    #[error("{0}")]
    UnexpectedCharacter(#[from] UnexpectedCharacter),

    /// A value couldn't be parsed
    #[error("Failed to parse value type: {0}")]
    ParseValue(Box<dyn StdError>),

    /// Unexpected end of string
    #[error("Unexpected end of string")]
    UnexpectedEnd,

    /// An unexpected token was encountered
    #[error("Unexpected token: {}", .got.displayable_type())]
    UnexpectedToken {
        /// The token which was encountered
        got: Token,

        /// The token variant which was expected
        ///
        /// (only the variant carries meaning, its data might be empty)
        exp: Token,
    },

    /// An unknown column was encountered
    #[error("Unknown column: {0}")]
    UnknownColumn(String),
}

/// OR expression
#[derive(Debug, Clone)]
pub struct Or<T>(
    /// List of expressions to be `or`ed
    pub Vec<And<T>>,
);

/// AND expression
#[derive(Debug, Clone)]
pub struct And<T>(
    /// List of expressions to be `and`ed
    pub Vec<Not<T>>,
);

/// Potentially negated expression
#[derive(Debug, Clone)]
pub struct Not<T> {
    /// Should the value be negated
    pub is_negated: bool,

    /// The leaf's value
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
