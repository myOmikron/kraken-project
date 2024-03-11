mod cursor;
mod value_parser;

use chrono::DateTime;
use chrono::Utc;
use ipnetwork::IpNetwork;

use self::cursor::Cursor;
use self::value_parser::parse_from_str;
use self::value_parser::parse_string;
use self::value_parser::wrap_maybe_range;
use self::value_parser::ValueParser;
use crate::modules::filter::lexer::tokenize;
use crate::modules::filter::lexer::Token;
use crate::modules::filter::parser::value_parser::parse_os_type;
use crate::modules::filter::parser::value_parser::parse_port_protocol;
use crate::modules::filter::parser::value_parser::parse_service_transport;
use crate::modules::filter::parser::value_parser::wrap_range;
use crate::modules::filter::And;
use crate::modules::filter::DomainAST;
use crate::modules::filter::GlobalAST;
use crate::modules::filter::HostAST;
use crate::modules::filter::Not;
use crate::modules::filter::Or;
use crate::modules::filter::ParseError;
use crate::modules::filter::PortAST;
use crate::modules::filter::ServiceAST;

impl GlobalAST {
    /// Parse a string into a [`GlobalAST`]
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(
            input,
            |GlobalAST { tags, created_at }, column, tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "createdAt" => parse_ast_field(
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
                 source_of,
                 source_of_tags,
                 source_of_created_at,
                 target_of,
                 target_of_tags,
                 target_of_created_at,
                 ips,
                 ips_created_at,
                 ips_tags,
                 ips_os,
             },
             column,
             tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "createdAt" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "domains" | "domain" => parse_ast_field(domains, tokens, parse_string),
                "sourceOf" => parse_ast_field(source_of, tokens, parse_string),
                "sourceOf.tags" | "sourceOf.tag" => {
                    parse_ast_field(source_of_tags, tokens, parse_string)
                }
                "sourceOf.createdAt" => parse_ast_field(
                    source_of_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "targetOf" => parse_ast_field(target_of, tokens, parse_string),
                "targetOf.tags" | "targetOf.tag" => {
                    parse_ast_field(target_of_tags, tokens, parse_string)
                }
                "targetOf.createdAt" => parse_ast_field(
                    target_of_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "ips.tags" | "ips.tag" | "ip.tags" | "ip.tag" => {
                    parse_ast_field(ips_tags, tokens, parse_string)
                }
                "ips.createdAt" | "ip.createdAt" => parse_ast_field(
                    ips_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips.os" | "ip.os" => parse_ast_field(ips_os, tokens, parse_os_type),
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
                 os,
                 created_at,
                 ports,
                 ports_created_at,
                 ports_protocols: ports_protocol,
                 ports_tags,
                 services,
                 services_ports,
                 services_protocols,
                 services_tags,
                 services_created_at,
                 services_transports,
                 domains,
                 domains_tags,
                 domains_created_at,
             },
             column,
             tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "createdAt" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "os" => parse_ast_field(os, tokens, parse_os_type),
                "ports" | "port" => {
                    parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>))
                }
                "ports.createdAt" | "port.createdAt" => parse_ast_field(
                    ports_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ports.protocols" | "ports.protocol" | "port.protocols" | "port.protocol" => {
                    parse_ast_field(ports_protocol, tokens, parse_port_protocol)
                }
                "ports.tags" | "ports.tag" | "port.tags" | "port.tag" => {
                    parse_ast_field(ports_tags, tokens, parse_string)
                }
                "services" | "service" => parse_ast_field(services, tokens, parse_string),
                "services.ports" | "services.port" | "service.ports" | "service.port" => {
                    parse_ast_field(
                        services_ports,
                        tokens,
                        wrap_maybe_range(parse_from_str::<u16>),
                    )
                }
                "services.protocols" | "services.protocol" | "service.protocols"
                | "service.protocol" => {
                    parse_ast_field(services_protocols, tokens, parse_port_protocol)
                }
                "services.tags" | "services.tag" | "service.tags" | "service.tag" => {
                    parse_ast_field(services_tags, tokens, parse_string)
                }
                "services.createdAt" | "service.createdAt" => parse_ast_field(
                    services_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "services.transports"
                | "service.transports"
                | "services.transport"
                | "service.transport" => {
                    parse_ast_field(services_transports, tokens, parse_service_transport)
                }
                "domains" | "domain" => parse_ast_field(domains, tokens, parse_string),
                "domains.tags" | "domains.tag" | "domain.tags" | "domain.tag" => {
                    parse_ast_field(domains_tags, tokens, parse_string)
                }
                "domains.createdAt" | "domain.createdAt" => parse_ast_field(
                    domains_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
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
                 ips_created_at,
                 ips_tags,
                 ips_os,
                 protocols,
                 created_at,
                 services,
                 services_tags,
                 services_created_at,
                 services_transports,
             },
             column,
             tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "createdAt" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ports" | "port" => {
                    parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>))
                }
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "ips.createdAt" | "ip.createdAt" => parse_ast_field(
                    ips_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips.tags" | "ips.tag" | "ip.tags" | "ip.tag" => {
                    parse_ast_field(ips_tags, tokens, parse_string)
                }
                "ips.os" | "ip.os" => parse_ast_field(ips_os, tokens, parse_os_type),
                "protocols" | "protocol" => parse_ast_field(protocols, tokens, parse_port_protocol),
                "services" | "service" => parse_ast_field(services, tokens, parse_string),
                "services.tags" | "services.tag" | "service.tags" | "service.tag" => {
                    parse_ast_field(services_tags, tokens, parse_string)
                }
                "services.createdAt" | "service.createdAt" => parse_ast_field(
                    services_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "services.transports"
                | "service.transports"
                | "services.transport"
                | "service.transport" => {
                    parse_ast_field(services_transports, tokens, parse_service_transport)
                }
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
                 ips_created_at,
                 ips_tags,
                 ips_os,
                 ports_tags,
                 ports_created_at,
                 protocols,
                 services,
                 ports,
                 transport,
             },
             column,
             tokens| match column {
                "tags" | "tag" => parse_ast_field(tags, tokens, parse_string),
                "createdAt" => parse_ast_field(
                    created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ips" | "ip" => parse_ast_field(ips, tokens, parse_from_str::<IpNetwork>),
                "ips.createdAt" | "ip.createdAt" => parse_ast_field(
                    ips_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "ip.tag" | "ip.tags" | "ips.tag" | "ips.tags" => {
                    parse_ast_field(ips_tags, tokens, parse_string)
                }
                "ips.os" | "ip.os" => parse_ast_field(ips_os, tokens, parse_os_type),
                "services" | "service" => parse_ast_field(services, tokens, parse_string),
                "ports" | "port" => {
                    parse_ast_field(ports, tokens, wrap_maybe_range(parse_from_str::<u16>))
                }
                "ports.createdAt" | "port.createdAt" => parse_ast_field(
                    ports_created_at,
                    tokens,
                    wrap_range(parse_from_str::<DateTime<Utc>>),
                ),
                "port.tag" | "port.tags" | "ports.tag" | "ports.tags" => {
                    parse_ast_field(ports_tags, tokens, parse_string)
                }
                "protocols" | "protocol" => parse_ast_field(protocols, tokens, parse_port_protocol),
                "transports" | "transport" => {
                    parse_ast_field(transport, tokens, parse_service_transport)
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
