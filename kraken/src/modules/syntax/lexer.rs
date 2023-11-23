use std::iter::Peekable;
use std::str::Chars;

use crate::modules::syntax::{LexerError, Token, TokenKind};

/// Main struct for tokenizing input
pub struct Lexer<'src> {
    input: &'src str,
    cursor: usize,
    current_char: Option<char>,
    peekable: Peekable<Chars<'src>>,
}

impl<'src> Lexer<'src> {
    /// create a new lexer for input
    pub fn new(input: &'src str) -> Self {
        Self {
            input,
            cursor: 0,
            current_char: None,
            peekable: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Result<Token<'src>, LexerError> {
        if self.cursor > self.input.len() {
            return Err(LexerError::Eof);
        }

        // Send EOF
        if self.cursor == self.input.len() {
            self.consume_char();
            return Ok(Token {
                kind: TokenKind::Eof,
                literal: "\0",
            });
        }

        let mut start = self.cursor;
        let mut end = 0;
        let mut token_kind = TokenKind::Bad;

        let Some(c) = self.consume_char() else {
            return Err(LexerError::UnexpectedEof);
        };

        if c == ',' {
            token_kind = TokenKind::LogicalOr;
        } else if c == ':' {
            token_kind = TokenKind::LogicalEqual;
        } else if c == '&' {
            token_kind = TokenKind::LogicalAnd;
        } else if c == '-' {
            token_kind = TokenKind::RangeOperator;
        } else if c == '!' {
            token_kind = TokenKind::LogicalNegate;
        } else if c == '\"' {
            token_kind = self.consume_date_time()?;
            start += 1;
            end += 1;
        } else if c.is_whitespace() {
            token_kind = TokenKind::WhiteSpace;
        } else if c.is_ascii_alphanumeric() {
            // limit to ascii
            token_kind = self.consume_alphanumeric(start)?;
        }

        end = self.cursor - end;

        // bigger than char boundary (ex: ☢)
        if c.len_utf8() > 1 {
            // implicit bad token
            let rest = c.len_utf8() - (end - start);
            end += rest;
            self.cursor += rest
        }
        let literal = &self.input[start..end];
        Ok(Token::new(token_kind, literal))
    }

    fn consume_char(&mut self) -> Option<char> {
        self.cursor += 1;
        self.current_char = self.peekable.next();
        self.current_char
    }

    fn consume_date_time(&mut self) -> Result<TokenKind, LexerError> {
        self.consume_date()?;

        let next = self.peek().ok_or(LexerError::UnexpectedEof)?;

        if next == '"' {
            self.consume_char();
            return Ok(TokenKind::Date);
        }

        if !next.is_whitespace() {
            return Err(LexerError::DateTimeFormat(Box::new(
                LexerError::UnexpectedSymbol {
                    expected: ' ',
                    found: next,
                    position: self.cursor,
                },
            )));
        }

        self.consume_until(|c| !c.is_whitespace());
        self.consume_time()?;
        self.consume_char();

        Ok(TokenKind::DateTime)
    }

    fn consume_time(&mut self) -> Result<(), LexerError> {
        let start = self.cursor;

        self.consume_until(|c| !c.is_ascii_digit());
        self.expect_char_at(':', start + 2)
            .map_err(|e| LexerError::DateTimeFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_digit());

        if self.cursor != start + 5 {
            return Err(LexerError::DateTimeFormat(Box::new(LexerError::TimeFormat)));
        }

        Ok(())
    }

    fn consume_date(&mut self) -> Result<(), LexerError> {
        let start = self.cursor;
        self.consume_until(|c| !c.is_ascii_digit());
        self.expect_char_at('-', start + 4)
            .map_err(|e| LexerError::DateTimeFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_digit());
        self.expect_char_at('-', start + 7)
            .map_err(|e| LexerError::DateTimeFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_digit());

        if self.cursor != start + 10 {
            return Err(LexerError::DateTimeFormat(Box::new(LexerError::DateFormat)));
        }

        Ok(())
    }

    fn consume_alphanumeric(&mut self, start: usize) -> Result<TokenKind, LexerError> {
        let c = self.current_char.unwrap();

        if c.is_ascii_digit() {
            // could be: Number | Uuid
            self.consume_until(|cur_c| !cur_c.is_ascii_digit());
            let Some(next) = self.peek() else {
                return Ok(TokenKind::Number);
            };

            if next.is_alphabetic() {
                // possible uuid
                return self.consume_uuid(start);
            }

            return Ok(TokenKind::Number);
        } else if c.is_alphabetic() {
            // could be: Word | Uuid

            self.consume_until(|cur_c| !cur_c.is_ascii_word());
            let Some(next) = self.peek() else {
                return Ok(TokenKind::Word);
            };

            if next.is_ascii_digit() {
                // possible Uuid
                return self.consume_uuid(start);
            }
            return Ok(TokenKind::Word);
        }

        unreachable!();
    }

    fn consume_uuid(&mut self, start: usize) -> Result<TokenKind, LexerError> {
        self.consume_until(|c| !c.is_ascii_alphanumeric());
        self.expect_char_at('-', start + 8)
            .map_err(|e| LexerError::UuidFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_alphanumeric());
        self.expect_char_at('-', start + 13)
            .map_err(|e| LexerError::UuidFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_alphanumeric());
        self.expect_char_at('-', start + 18)
            .map_err(|e| LexerError::UuidFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_alphanumeric());
        self.expect_char_at('-', start + 23)
            .map_err(|e| LexerError::UuidFormat(Box::new(e)))?;
        self.consume_char();

        self.consume_until(|c| !c.is_ascii_alphanumeric());

        if self.cursor != start + 36 {
            return Err(LexerError::UuidFormat(Box::new(LexerError::UuidLength(
                self.cursor - start,
            ))));
        }

        Ok(TokenKind::Uuid)
    }

    fn consume_until(&mut self, condition: impl Fn(char) -> bool) {
        while let Some(cc) = self.peek() {
            if condition(cc) {
                break;
            }
            self.consume_char();
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.peekable.peek().cloned()
    }

    fn expect_char_at(&mut self, expected: char, pos: usize) -> Result<(), LexerError> {
        let c = self.peek().ok_or(LexerError::Eof)?;

        if c != expected || self.cursor != pos {
            return Err(LexerError::UnexpectedSymbol {
                expected,
                found: c,
                position: pos,
            });
        }

        Ok(())
    }
}

trait AsciiWord {
    fn is_ascii_word(&self) -> bool;
}

impl AsciiWord for char {
    fn is_ascii_word(&self) -> bool {
        self.is_ascii_alphabetic() && self != &':' || self == &'_'
    }
}
