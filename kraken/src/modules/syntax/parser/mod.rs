mod cursor;
mod value_parser;

use std::error::Error as StdError;

use thiserror::Error;

use self::cursor::Cursor;
use self::value_parser::{parse_from_str, parse_string, wrap_maybe_range, ValueParser};
use super::{tokenize, And, Not, Or, PortAST, Token, UnexpectedCharacter};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{0}")]
    UnexpectedCharacter(#[from] UnexpectedCharacter),

    #[error("Failed to parse value type: {0}")]
    ParseValue(Box<dyn StdError>),

    #[error("Unexpected end of string")]
    UnexpectedEnd,

    #[error("Unexpected token: {}", .got.displayable_type())]
    UnexpectedToken { got: Token, exp: Token },

    #[error("Unknown column: {0}")]
    UnknownColumn(String),
}

impl PortAST {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_ast(input, |ast: &mut PortAST, column, tokens| match column {
            "tags" => {
                ast.tags
                    .get_or_insert(Or(Vec::new()))
                    .0
                    .extend(parse_or(tokens, parse_string)?.0);
                Ok(())
            }
            "ports" => {
                ast.ports
                    .get_or_insert(Or(Vec::new()))
                    .0
                    .extend(parse_or(tokens, wrap_maybe_range(parse_from_str::<u16>))?.0);
                Ok(())
            }
            _ => Err(ParseError::UnknownColumn(column.to_string())),
        })
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
