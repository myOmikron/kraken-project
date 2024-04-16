use std::str::FromStr;

use serde::de::StdError;

use crate::models::OsType;
use crate::models::PortProtocol;
use crate::modules::filter::lexer::Token;
use crate::modules::filter::parser::cursor::Cursor;
use crate::modules::filter::MaybeRange;
use crate::modules::filter::ParseError;
use crate::modules::filter::Range;
use crate::modules::filter::ServiceTransport;

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
    // don't forget to update docs/user/filter.md and frontend if you extend this!
    if string.eq_ignore_ascii_case("tcp") {
        Ok(PortProtocol::Tcp)
    } else if string.eq_ignore_ascii_case("udp") {
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

/// Parse a single [`ServiceTransport`]
pub fn parse_service_transport(tokens: &mut Cursor) -> Result<ServiceTransport, ParseError> {
    let string = tokens.next_value()?;
    // don't forget to update docs/user/filter.md and frontend if you extend this!
    if string.eq_ignore_ascii_case("raw") {
        Ok(ServiceTransport::Raw)
    } else if string.eq_ignore_ascii_case("tls") {
        Ok(ServiceTransport::Tls)
    } else {
        Err(ParseError::ParseValue(
            format!("Unknown service transport: {string}").into(),
        ))
    }
}

/// Parse a single [`OsType`]
pub fn parse_os_type(tokens: &mut Cursor) -> Result<OsType, ParseError> {
    let string = tokens.next_value()?;
    // don't forget to update docs/user/filter.md and frontend if you extend this!
    if string.eq_ignore_ascii_case("unknown") {
        Ok(OsType::Unknown)
    } else if string.eq_ignore_ascii_case("linux") {
        Ok(OsType::Linux)
    } else if string.eq_ignore_ascii_case("windows") {
        Ok(OsType::Windows)
    } else if string.eq_ignore_ascii_case("apple") {
        Ok(OsType::Apple)
    } else if string.eq_ignore_ascii_case("android") {
        Ok(OsType::Android)
    } else if string.eq_ignore_ascii_case("freebsd") {
        Ok(OsType::FreeBSD)
    } else {
        Err(ParseError::ParseValue(
            format!("Unknown OS type: {string}").into(),
        ))
    }
}

/// Parse a boolean (yes/true or no/false)
pub fn parse_boolean(tokens: &mut Cursor) -> Result<bool, ParseError> {
    let string = tokens.next_value()?;
    // don't forget to update docs/user/filter.md and frontend if you extend this!
    if string.eq_ignore_ascii_case("yes") || string.eq_ignore_ascii_case("true") {
        Ok(true)
    } else if string.eq_ignore_ascii_case("no") || string.eq_ignore_ascii_case("false") {
        Ok(false)
    } else {
        Err(ParseError::ParseValue(
            format!("Expected yes/no/true/false, not: {string}").into(),
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
        let start = if matches!(tokens.peek_token(), Some(Token::RangeOperator)) {
            None
        } else {
            Some(parse_value(tokens)?)
        };
        match tokens.next_token() {
            Some(Token::RangeOperator) => {}
            Some(token) => {
                return Err(ParseError::UnexpectedToken {
                    got: token.clone(),
                    exp: Token::RangeOperator,
                })
            }
            None => return Err(ParseError::UnexpectedEnd),
        }

        let mut tokens_clone = tokens.clone();
        let end = parse_value(&mut tokens_clone).ok();
        if end.is_some() {
            *tokens = tokens_clone;
        }

        Ok(Range { start, end })
    }
}
