//! The code for the filter syntax parser

use std::any::Any;
use std::cell::RefCell;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use uuid::Uuid;

use crate::modules::syntax::{And, Lexer, MaybeRange, Not, Or, ParserError, Token, TokenKind};

struct Inner<'src> {
    lexer: Lexer<'src>,
    current: Token<'src>,
    look_ahead: Token<'src>,
    not: bool,
    ranged: bool,
    seen_range_operator: bool,
    field: Option<&'src str>,
}

/// Main struct for parsing
pub struct Parser<'src>(RefCell<Inner<'src>>);

impl<'src> Parser<'src> {
    /// Needs at least 2 tokens or errors out
    pub fn new(input: &'src str) -> Result<Parser, ParserError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token().map_err(ParserError::TokenError)?;
        let look_ahead = lexer.next_token().map_err(ParserError::TokenError)?;

        Ok(Self(RefCell::new(Inner {
            lexer,
            current,
            look_ahead,
            not: false,
            ranged: false,
            seen_range_operator: false,
            field: None,
        })))
    }

    // TODO: return struct with 'parse_statement'
    /// parse next field
    ///
    /// if 'parse_statement' is not called returns previous field
    pub fn parse_field(&self) -> Result<Option<String>, ParserError> {
        if self.is_eof() {
            return Ok(None);
        }

        if self.0.borrow().field.is_some() {
            return Ok(Some(self.0.borrow().field.unwrap().to_string()));
        }

        let field = self.current_token();
        if field.kind != TokenKind::Word {
            return Err(ParserError::SyntaxError(String::from("expecting field")));
        }

        let equal_token = self.consume_token()?;
        if equal_token.kind != TokenKind::LogicalEqual {
            return Err(ParserError::SyntaxError(String::from("expecting ':'")));
        }

        self.consume_token()?;

        self.0.borrow_mut().field = Some(field.literal);

        Ok(Some(field.literal.to_string()))
    }

    /// parse one statement at a time
    ///
    /// must be polled to get all statements
    ///
    /// end on [`Ok(None)`] or [`Err(String)`]
    pub fn parse_statement(&self, ranged: bool) -> Result<Option<Box<dyn Any>>, ParserError> {
        if self.is_eof() {
            return Ok(None);
        }

        let _ = self.0.borrow_mut().field.take();

        if ranged {
            self.0.borrow_mut().ranged = ranged;

            Ok(Some(self.parse_by_range_type()?))
        } else {
            Ok(Some(self.parse_by_type()?))
        }
    }

    fn parse_by_range_type(&self) -> Result<Box<dyn Any>, ParserError> {
        let token = self.current_token();
        match token.kind {
            TokenKind::Number => Ok(Box::new(self.parse_or_statement::<MaybeRange<i64>>()?)),
            TokenKind::Date | TokenKind::DateTime => Ok(Box::new(
                self.parse_or_statement::<MaybeRange<DateTime<Utc>>>()?,
            )),
            TokenKind::RangeOperator => {
                self.consume_token()?;
                self.set_state_seen_range(true);
                self.parse_by_range_type()
            }
            TokenKind::LogicalNegate => {
                self.consume_token()?;
                self.set_state_not(true);
                self.parse_by_range_type()
            }
            _ => Err(ParserError::UnexpectedToken(token.literal.to_string())),
        }
    }

    fn parse_by_type(&self) -> Result<Box<dyn Any>, ParserError> {
        let token = self.current_token();
        let or: Box<dyn Any> = match token.kind {
            TokenKind::Word => Box::new(self.parse_or_statement::<String>()?),
            TokenKind::Number => Box::new(self.parse_or_statement::<i64>()?),
            TokenKind::Uuid => Box::new(self.parse_or_statement::<Uuid>()?),
            TokenKind::Date | TokenKind::DateTime => {
                Box::new(self.parse_or_statement::<DateTime<Utc>>()?)
            }
            TokenKind::LogicalNegate => {
                self.consume_token()?;
                self.set_state_not(true);
                return self.parse_by_type();
            }
            _ => return Err(ParserError::UnexpectedToken(token.literal.to_string())),
        };
        Ok(or)
    }

    fn parse_or_statement<T: Clone + 'static>(&self) -> Result<Or<T>, ParserError> {
        let mut or = Or(vec![]);
        while !self.is_eof() && !self.is_next_statement() {
            or.0.push(self.parse_and_statement()?);
            let token = self.current_token();
            if token.kind == TokenKind::LogicalOr {
                self.consume_token()?;
            }
        }

        Ok(or)
    }

    fn parse_and_statement<T: Clone + 'static>(&self) -> Result<And<T>, ParserError> {
        let mut and = And(vec![]);
        while !self.is_eof() && !self.is_next_statement() {
            let token = self.current_token();
            if token.kind == TokenKind::LogicalOr {
                self.consume_token()?;
                break;
            } else if token.kind == TokenKind::LogicalAnd {
                self.consume_token()?;
                continue;
            } else if token.kind == TokenKind::LogicalNegate {
                self.set_state_not(true);
                self.consume_token()?;
            }

            and.0.push(self.parse_not_statement()?);
        }

        Ok(and)
    }

    fn parse_not_statement<T: Clone + 'static>(&self) -> Result<Not<T>, ParserError> {
        let value = self.parse_basic_statement()?;

        if self.get_state_not() {
            self.set_state_not(false);
            Ok(Not(true, value))
        } else {
            Ok(Not(false, value))
        }
    }

    fn parse_basic_statement<T: Clone + 'static>(&self) -> Result<T, ParserError> {
        let dyn_value = self.parse_token_value()?;
        let typed_value = dyn_value
            .downcast_ref::<T>()
            .ok_or(ParserError::MixedTypes)?
            .to_owned();
        Ok(typed_value)
    }

    fn parse_token_value(&self) -> Result<Box<dyn Any>, ParserError> {
        let token = self.current_token();
        match token.kind {
            TokenKind::Word => {
                self.consume_token()?;
                Ok(Box::new(token.literal.to_string()))
            }
            TokenKind::Number => {
                self.consume_token()?;

                let num = i64::from_str(token.literal).map_err(ParserError::Number)?;

                self.parse_range_if_needed(num, |kind| kind == TokenKind::Number)
            }
            TokenKind::Uuid => {
                self.consume_token()?;

                let uuid = Uuid::from_str(token.literal).map_err(ParserError::Uuid)?;

                Ok(Box::new(uuid))
            }
            TokenKind::Date => {
                self.consume_token()?;

                let date = NaiveDate::parse_from_str(token.literal, "%Y-%m-%d")
                    .map_err(ParserError::Date)?
                    .and_time(Default::default())
                    .and_utc();

                self.parse_range_if_needed(date, |kind| {
                    kind == TokenKind::Date || kind == TokenKind::DateTime
                })
            }
            TokenKind::DateTime => {
                self.consume_token()?;

                let date_time = NaiveDateTime::parse_from_str(token.literal, "%Y-%m-%d %H:%M")
                    .map_err(ParserError::Date)?
                    .and_utc();

                self.parse_range_if_needed(date_time, |kind| {
                    kind == TokenKind::Date || kind == TokenKind::DateTime
                })
            }
            TokenKind::RangeOperator => {
                if !self.is_ranged() {
                    return Err(ParserError::UnexpectedToken(token.literal.to_string()));
                }
                self.consume_token()?;
                self.set_state_seen_range(true);
                self.parse_token_value()
            }
            _ => Err(ParserError::UnexpectedToken(token.literal.to_string())),
        }
    }

    fn parse_range_if_needed<T: Clone + 'static>(
        &self,
        value: T,
        is_same: fn(TokenKind) -> bool,
    ) -> Result<Box<dyn Any>, ParserError> {
        if !self.is_ranged() {
            return Ok(Box::new(value));
        }

        if self.get_state_seen_range() {
            self.set_state_seen_range(false);
            return Ok(Box::new(MaybeRange::Range {
                start: None,
                end: Some(value),
            }));
        }

        if self.current_token().kind == TokenKind::RangeOperator {
            self.consume_token()?;
            let mut end = None;
            let match_token = self.current_token();
            if is_same(match_token.kind) {
                {
                    self.0.borrow_mut().ranged = false;
                }
                let dyn_value = self.parse_token_value()?;
                let typed_value = dyn_value
                    .downcast_ref::<T>()
                    .ok_or(ParserError::MixedTypes)?
                    .to_owned();

                end = Some(typed_value);

                {
                    self.0.borrow_mut().ranged = true;
                }
            }

            match match_token.kind {
                TokenKind::RangeOperator | TokenKind::LogicalEqual | TokenKind::LogicalNegate => {
                    return Err(ParserError::UnexpectedToken(
                        match_token.literal.to_string(),
                    ));
                }
                _ => {}
            }

            let range = MaybeRange::Range {
                start: Some(value),
                end,
            };

            return Ok(Box::new(range));
        }

        Ok(Box::new(MaybeRange::Single(value)))
    }

    fn consume_token(&self) -> Result<Token<'src>, ParserError> {
        let mut inner = self.0.borrow_mut();
        inner.current = inner.look_ahead;

        if inner.look_ahead.kind != TokenKind::Eof {
            loop {
                let token = inner.lexer.next_token().map_err(ParserError::TokenError)?;
                if self.ignore_token(&token) {
                    continue;
                } else {
                    inner.look_ahead = token;
                    break;
                }
            }
        }

        Ok(inner.current)
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.0.borrow().current.kind == TokenKind::Eof
    }

    #[inline]
    fn is_next_statement(&self) -> bool {
        self.0.borrow().look_ahead.kind == TokenKind::LogicalEqual
    }

    #[inline]
    fn ignore_token(&self, token: &Token) -> bool {
        token.kind == TokenKind::WhiteSpace
    }

    #[inline]
    fn current_token(&self) -> Token<'src> {
        self.0.borrow().current
    }

    #[inline]
    fn set_state_not(&self, not: bool) {
        self.0.borrow_mut().not = not;
    }

    #[inline]
    fn set_state_seen_range(&self, seen: bool) {
        self.0.borrow_mut().seen_range_operator = seen;
    }

    #[inline]
    fn get_state_not(&self) -> bool {
        self.0.borrow().not
    }

    #[inline]
    fn get_state_seen_range(&self) -> bool {
        self.0.borrow().seen_range_operator
    }

    #[inline]
    fn is_ranged(&self) -> bool {
        self.0.borrow().ranged
    }
}
