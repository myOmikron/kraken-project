mod cursor;
mod value_parser;

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;

use self::cursor::Cursor;
use self::value_parser::{parse_from_str, parse_string, wrap_maybe_range, ValueParser};
use crate::modules::filter::lexer::{tokenize, Token};
use crate::modules::filter::parser::value_parser::{parse_port_protocol, wrap_range};
use crate::modules::filter::{
    And, DomainAST, GlobalAST, HostAST, Not, Or, ParseError, PortAST, ServiceAST,
};

impl GlobalAST {
    /// Parse a string into a [`GlobalAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |GlobalAST { tags, created_at }, column, tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
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
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "domains" | "domain" => parse_ast_field(domains, tokens, parse_string),
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
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
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
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ports" | "port" => {
                    parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>))
                }
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "protocols" | "protocol" => parse_ast_field(protocols, tokens, parse_port_protocol),
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
                 services,
                 ports,
             },
             column,
             tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "created_at" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "services" | "service" => parse_ast_field(services, tokens, parse_string),
                "ports" | "port" => {
                    parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>))
                }
                _ => Err(ParseError::UnknownColumn(column.to_string())),
            },
        )
    }
}

/// Helper function to be called from `...AST::parse`
///
/// ## Arguments
/// - `input` is the source string to parse
/// - `parse_column` is a callback which is invoked with each column which is encountered.
///     Its arguments are the ast being constructed, the column's name and the cursor to parse the column's expression.
pub fn parse_ast<A: Default>(
    input: &str,
    parse_column: impl Fn(&mut A, &str, &mut Cursor) -> Result<(), ParseError>,
) -> Result<A, ParseError> {
    let tokens = tokenize(input)?;
    let mut tokens = Cursor::new(&tokens);

    let mut ast = A::default();
    while let Some(token) = tokens.next_token() {
        match token {
            Token::Column(column) => parse_column(&mut ast, column, &mut tokens)?,
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

/// Helper function to be called in `parse_ast`'s callback.
///
/// It parses an expression using a [`ValueParser`] to parse the leaves
/// and adds the result to the ast under construction.
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

/// Parse an [`Or`] expression using a [`ValueParser`] to parse the leaves
pub fn parse_or<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<Or<T>, ParseError> {
    let mut list = vec![parse_and(tokens, parse_value)?];
    while matches!(tokens.peek_token(), Some(Token::LogicalOr)) {
        let _ = tokens.next_token(); // Consume the ','
        list.push(parse_and(tokens, parse_value)?);
    }
    Ok(Or(list))
}

/// Parse an [`And`] expression using a [`ValueParser`] to parse the leaves
pub fn parse_and<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<And<T>, ParseError> {
    let mut list = vec![parse_not(tokens, parse_value)?];
    while matches!(tokens.peek_token(), Some(Token::LogicalAnd)) {
        let _ = tokens.next_token(); // Consume the '&'
        list.push(parse_not(tokens, parse_value)?);
    }
    Ok(And(list))
}

/// Parse a [`Not`] expression using a [`ValueParser`] to parse the potentially negated value
pub fn parse_not<T>(
    tokens: &mut Cursor,
    parse_value: impl ValueParser<T>,
) -> Result<Not<T>, ParseError> {
    let mut is_negated = false;
    if matches!(tokens.peek_token(), Some(Token::LogicalNot)) {
        let _ = tokens.next_token(); // Consume the '!'
        is_negated = true;
    }
    Ok(Not {
        is_negated,
        value: parse_value(tokens)?,
    })
}
