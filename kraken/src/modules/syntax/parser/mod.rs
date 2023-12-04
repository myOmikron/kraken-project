mod cursor;
mod value_parser;

use std::error::Error as StdError;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use thiserror::Error;

use self::cursor::Cursor;
use self::value_parser::{parse_from_str, parse_string, wrap_maybe_range, ValueParser};
use super::{
    tokenize, And, DomainAST, GlobalAST, HostAST, Not, Or, PortAST, ServiceAST, Token,
    UnexpectedCharacter,
};
use crate::modules::syntax::parser::value_parser::{parse_port_protocol, wrap_range};

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

impl GlobalAST {
    /// Parse a string into a [`GlobalAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |GlobalAST { tags, created_at }, column, tokens| match column {
                "tags" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

impl DomainAST {
    /// Parse a string into a [`DomainAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |DomainAST {
                 tags,
                 domains,
                 created_at,
             },
             column,
             tokens| match column {
                "tags" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "domains" => parse_ast_field(domains, tokens, parse_string),
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

impl HostAST {
    /// Parse a string into a [`HostAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |HostAST {
                 tags,
                 ips,
                 created_at,
             },
             column,
             tokens| match column {
                "tags" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

impl PortAST {
    /// Parse a string into a [`PortAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |PortAST {
                 tags,
                 ports,
                 ips,
                 protocols,
                 created_at,
             },
             column,
             tokens| match column {
                "tags" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ports" => parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>)),
                "ips" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "protocols" => parse_ast_field(protocols, tokens, parse_port_protocol),
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

impl ServiceAST {
    /// Parse a string into a [`ServiceAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |ServiceAST {
                 tags,
                 created_at,
                 ips,
                 names,
                 ports,
             },
             column,
             tokens| match column {
                "tags" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "names" => parse_ast_field(names, tokens, parse_string),
                "ports" => parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>)),
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

pub fn parse_ast<A: Default>(
    input: &str,
    parse_column: impl Fn(&mut A, &str, &mut Cursor) -> Result<(), ParseError>,
) -> Result<A, ParseError> {
    let tokens = tokenize(input)?;
    let mut tokens = Cursor::new(&tokens);

    let mut ast = A::default();
    while let Ok(token) = tokens.next_token() {
        match token {
            Token::Column(column) => parse_column(&mut ast, &column, &mut tokens)?,
            token => {
                return Err(ParseError::UnexpectedToken {
                    got: token.clone(),
                    exp: Token::Column(String::new()),
                })
            }
        }
    }
    Ok(ast)
}

pub fn parse_ast_field<T>(
    ast_field: &mut Option<Or<T>>,
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<(), ParseError> {
    ast_field
        .get_or_insert(Or(Vec::new()))
        .0
        .extend(parse_or(tokens, parse_value)?.0);
    Ok(())
}

pub fn parse_or<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<Or<T>, ParseError> {
    let mut list = vec![parse_and(tokens, parse_value)?];
    while matches!(tokens.peek_token(), Ok(Token::LogicalOr)) {
        let _ = tokens.next_token(); // Consume the ','
        list.push(parse_and(tokens, parse_value)?);
    }
    Ok(Or(list))
}

pub fn parse_and<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<And<T>, ParseError> {
    let mut list = vec![parse_not(tokens, parse_value)?];
    while matches!(tokens.peek_token(), Ok(Token::LogicalAnd)) {
        let _ = tokens.next_token(); // Consume the '&'
        list.push(parse_not(tokens, parse_value)?);
    }
    Ok(And(list))
}

pub fn parse_not<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<Not<T>, ParseError> {
    let mut is_negated = false;
    if matches!(tokens.peek_token()?, Token::LogicalNot) {
        let _ = tokens.next_token(); // Consume the '&'
        is_negated = true;
    }
    Ok(Not {
        is_negated,
        value: parse_value(tokens)?,
    })
}
