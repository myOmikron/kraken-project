use std::iter::Peekable;
use std::slice;

use crate::modules::filter::lexer::Token;
use crate::modules::filter::ParseError;

/// An iterator over [`Token`] specialized for our parser
///
/// - use `next_value` as shorthand for `next_token` with the additional check
#[derive(Clone)]
pub struct Cursor<'a>(Peekable<slice::Iter<'a, Token>>);

impl<'a> Cursor<'a> {
    /// Construct a cursor from a slice of tokens
    pub fn new(slice: &'a [Token]) -> Self {
        Self(slice.iter().peekable())
    }

    /// Yield the next token without advancing the cursor
    pub fn peek_token(&mut self) -> Option<&'a Token> {
        self.0.peek().copied()
    }

    /// Yield the next token and advance the cursor
    pub fn next_token(&mut self) -> Option<&'a Token> {
        self.0.next()
    }

    /// Yield the next token, check it to be a [`Token::Value`] and advance the cursor
    ///
    /// ## Errors
    /// - with [`ParseError::UnexpectedEnd`] if all tokens have been consumed
    /// - with [`ParseError::UnexpectedToken`] if the next token was not a [`Token::Value`]
    pub fn next_value(&mut self) -> Result<&'a String, ParseError> {
        match self.next_token().ok_or(ParseError::UnexpectedEnd)? {
            Token::Value(string) => Ok(string),
            token => Err(ParseError::UnexpectedToken {
                got: token.clone(),
                exp: Token::Value(String::new()),
            }),
        }
    }
}
