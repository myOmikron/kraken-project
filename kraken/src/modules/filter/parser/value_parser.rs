use std::str::FromStr;

use serde::de::StdError;

use crate::models::PortProtocol;
use crate::modules::filter::lexer::Token;
use crate::modules::filter::parser::cursor::Cursor;
use crate::modules::filter::{MaybeRange, ParseError, Range};

/// Trait alias for `Fn(&mut Cursor) -> Result<T, ParseError>` and `Copy`
pub trait ValueParser<T>: Fn(&mut Cursor) -> Result<T, ParseError> + Copy {}
impl<T, F: Fn(&mut Cursor) -> Result<T, ParseError> + Copy> ValueParser<T> for F {}

/// Parse a single string
pub fn parse_string(tokens: &mut Cursor) -> Result<String, ParseError> {
    tokens.next_value().cloned()
}

/// Parse a single value which can be converted from a string using [`FromStr`]
pub fn parse_from_str<T: FromStr>(tokens: &mut Cursor) -> Result<T, ParseError>
where
    T::Err: StdError + 'static,
{
    tokens
        .next_value()?
        .parse()
        .map_err(|error| ParseError::ParseValue(Box::new(error)))
}

/// Parse a single [`PortProtocol`]
pub fn parse_port_protocol(tokens: &mut Cursor) -> Result<PortProtocol, ParseError> {
    let string = tokens.next_value()?;
    if string.eq_ignore_ascii_case("tcp") {
        Ok(PortProtocol::Tcp)
    } else if string.eq_ignore_ascii_case("ucp") {
        Ok(PortProtocol::Udp)
    } else if string.eq_ignore_ascii_case("sctp") {
        Ok(PortProtocol::Sctp)
    } else if string.eq_ignore_ascii_case("unknown") {
        Ok(PortProtocol::Unknown)
    } else {
        Err(ParseError::ParseValue(
            format!("Unknown port protocol: {string}").into(),
        ))
    }
}

/// Wraps a [`ValueParser<T>`] to produce a [`ValueParser<MaybeRange<T>>`]
pub fn wrap_maybe_range<T>(parse_value: impl ValueParser<T>) -> impl ValueParser<MaybeRange<T>> {
    let parse_range = wrap_range(parse_value);
    move |tokens: &mut Cursor| {
        let mut tokens_clone = tokens.clone();
        if let Ok(range) = parse_range(&mut tokens_clone) {
            *tokens = tokens_clone;
            Ok(MaybeRange::Range(range))
        } else {
            parse_value(tokens).map(MaybeRange::Single)
        }
    }
}

/// Wraps a [`ValueParser<T>`] to produce a [`ValueParser<Range<T>>`]
pub fn wrap_range<T>(parse_value: impl ValueParser<T>) -> impl ValueParser<Range<T>> {
    move |tokens: &mut Cursor| {
        let start = if matches!(tokens.peek_token()?, Token::RangeOperator) {
            None
        } else {
            Some(parse_value(tokens)?)
        };
        match tokens.next_token()? {
            Token::RangeOperator => {}
            token => {
                return Err(ParseError::UnexpectedToken {
                    got: token.clone(),
                    exp: Token::RangeOperator,
                })
            }
        }

        let mut tokens_clone = tokens.clone();
        let end = parse_value(&mut tokens_clone).ok();
        if end.is_some() {
            *tokens = tokens_clone;
        }

        Ok(Range { start, end })
    }
}
