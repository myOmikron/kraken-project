//! This module holds all the code for lexing & parsing the filter syntax

use std::error::Error as StdError;
use std::fmt::Debug;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;
use thiserror::Error;

use crate::models::OsType;
use crate::models::PortProtocol;
use crate::modules::filter::lexer::Token;
use crate::modules::filter::lexer::UnexpectedCharacter;

mod lexer;
mod parser;
mod sqler;

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

    /// Filter domains by their targets
    pub source_of: Option<Or<String>>,

    /// Filter domains by their targets' tags
    pub source_of_tags: Option<Or<String>>,

    /// Filter domains by their targets' creation time
    pub source_of_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter domains by their sources
    pub target_of: Option<Or<String>>,

    /// Filter domains by their sources' tags
    pub target_of_tags: Option<Or<String>>,

    /// Filter domains by their sources' creation time
    pub target_of_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter domains by their hosts
    pub ips: Option<Or<IpNetwork>>,

    /// Filter domains by their hosts' creation time
    pub ips_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter domains by their hosts' tags
    pub ips_tags: Option<Or<String>>,

    /// Filter domains by their hosts' OS
    pub ips_os: Option<Or<OsType>>,
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

    /// Filter hosts by their OS
    pub os: Option<Or<OsType>>,

    /// Filter hosts by their ports
    pub ports: Option<Or<MaybeRange<u16>>>,

    /// Filter hosts by their ports' protocols
    pub ports_protocols: Option<Or<PortProtocol>>,

    /// Filter hosts by their ports' tags
    pub ports_tags: Option<Or<String>>,

    /// Filter hosts by their ports' creation time
    pub ports_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter hosts by their services
    pub services: Option<Or<String>>,

    /// Filter hosts by their services' ports
    pub services_ports: Option<Or<MaybeRange<u16>>>,

    /// Filter hosts by their services' protocols
    pub services_protocols: Option<Or<PortProtocol>>,

    /// Filter hosts by their services' tags
    pub services_tags: Option<Or<String>>,

    /// Filter hosts by their services' creation time
    pub services_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter ports by their services' transport type
    pub services_transports: Option<Or<ServiceTransport>>,

    /// Filter hosts by their domains' tags
    pub domains: Option<Or<String>>,

    /// Filter hosts by their domains
    pub domains_tags: Option<Or<String>>,

    /// Filter hosts by their domains
    pub domains_created_at: Option<Or<Range<DateTime<Utc>>>>,
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

    /// Filter ports by their hosts' creation time
    pub ips_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter ports by their hosts' tags
    pub ips_tags: Option<Or<String>>,

    /// Filter ports by their hosts' OS
    pub ips_os: Option<Or<OsType>>,

    /// Filter by protocols
    pub protocols: Option<Or<PortProtocol>>,

    /// Filter ports by their services
    pub services: Option<Or<String>>,

    /// Filter ports by their services' tags
    pub services_tags: Option<Or<String>>,

    /// Filter ports by their services' creation time
    pub services_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter ports by their services' transport type
    pub services_transports: Option<Or<ServiceTransport>>,
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

    /// Filter services by their hosts' creation time
    pub ips_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter services by their hosts' tags
    pub ips_tags: Option<Or<String>>,

    /// Filter services by their hosts' OS
    pub ips_os: Option<Or<OsType>>,

    /// Filter by ports
    pub ports: Option<Or<MaybeRange<u16>>>,

    /// Filter services by their ports' tags
    pub ports_tags: Option<Or<String>>,

    /// Filter services by their ports' creation time
    pub ports_created_at: Option<Or<Range<DateTime<Utc>>>>,

    /// Filter services by their ports' protocols
    pub protocols: Option<Or<PortProtocol>>,

    /// Filter by service name
    pub services: Option<Or<String>>,

    /// Filter services by their transport types
    pub transport: Option<Or<ServiceTransport>>,
}

/// Service transport protocol. See `protocols` field in [crate::models::Service].
#[derive(Debug)]
pub enum ServiceTransport {
    /// Raw unencrypted traffic
    Raw,
    /// TLS encrypted traffic
    Tls,
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
