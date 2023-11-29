use std::iter::Peekable;
use std::slice;

use super::super::Token;
use super::ParseError;

#[derive(Clone)]
pub struct Cursor<'a>(Peekable<slice::Iter<'a, Token>>);

impl<'a> Cursor<'a> {
    /// Construct a cursor from a slice of tokens
    pub fn new(slice: &'a [Token]) -> Self {
        Self(slice.iter().peekable())
    }

    /// Yield the next token without advancing the cursor
    ///
    /// ## Errors
    /// - with [`ParseError::UnexpectedEnd`] if all tokens have been consumed
    pub fn peek_token(&mut self) -> Result<&'a Token, ParseError> {
        self.0.peek().copied().ok_or(ParseError::UnexpectedEnd)
    }

    /// Yield the next token and advance the cursor
    ///
    /// ## Errors
    /// - with [`ParseError::UnexpectedEnd`] if all tokens have been consumed
    pub fn next_token(&mut self) -> Result<&'a Token, ParseError> {
        self.0.next().ok_or(ParseError::UnexpectedEnd)
    }

    /// Yield the next token, check it to be a [`Token::Value`] and advance the cursor
    ///
    /// ## Errors
    /// - with [`ParseError::UnexpectedEnd`] if all tokens have been consumed
    /// - with [`ParseError::UnexpectedToken`] if the next token was not a [`Token::Value`]
    pub fn next_value(&mut self) -> Result<&'a String, ParseError> {
        match self.next_token()? {
            Token::Value(string) => Ok(string),
            token => Err(ParseError::UnexpectedToken {
                got: token.clone(),
                exp: Token::Value(String::new()),
            }),
        }
    }
}
