//! This module holds all the error code for lexer and parser

use std::num::ParseIntError;

use chrono::ParseError;
use thiserror::Error;

/// Errors that occurs in the lexer
#[derive(Error, Debug)]
pub enum LexerError {
    /// Unexpected symbol
    #[error("expecting symbol '{expected}' but found '{found}' @ position {position}")]
    UnexpectedSymbol {
        /// expected symbol
        expected: char,
        /// found symbol
        found: char,
        /// position in input
        position: usize,
    },

    /// uuid format error
    #[error("Error in Uuid format: {0}")]
    UuidFormat(Box<LexerError>),

    /// uuid length error
    #[error("Uuid wrong length: expect '36' found '{0}'")]
    UuidLength(usize),

    /// date or time error
    #[error("DateTime format error: {0}")]
    DateTimeFormat(Box<LexerError>),

    /// date format error
    #[error("Date format YYYY-mm-dd")]
    DateFormat,

    /// time format error
    #[error("Time format HH:MM")]
    TimeFormat,

    /// unexpected end of input
    #[error("unexpected EOF")]
    UnexpectedEof,

    /// end of input
    #[error("EOF")]
    Eof,
}

/// errors during parsing
#[derive(Error, Debug)]
pub enum ParserError {
    /// the error from the the lexer
    #[error("error getting token: {0}")]
    TokenError(LexerError),

    /// error in syntax
    #[error("syntax error: {0}")]
    SyntaxError(String),

    /// unexpected token
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),

    /// type error
    #[error("type error, cannot mix types")]
    MixedTypes,

    /// parsing date error
    #[error("error parsing date: {0}")]
    Date(ParseError),

    /// parsing number error
    #[error("error parsing number: {0}")]
    Number(ParseIntError),

    /// uuid parsing error
    #[error("error parsing uuid: {0}")]
    Uuid(uuid::Error),
}
