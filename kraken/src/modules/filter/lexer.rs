use thiserror::Error;

/// The tokens produced by [`tokenize`]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Any column name
    Column(String),
    /// Any value
    Value(String),

    /// `,`
    LogicalOr,
    /// `&`
    LogicalAnd,
    /// `!`
    LogicalNot,
    /// `-`
    RangeOperator,
}

impl Token {
    /// Get the variant's name as string
    pub fn displayable_type(&self) -> &'static str {
        match self {
            Self::Column(_) => "column name",
            Self::Value(_) => "value",
            Self::LogicalOr => "or",
            Self::LogicalAnd => "and",
            Self::LogicalNot => "not",
            Self::RangeOperator => "range",
        }
    }
}

/// Split an input string into tokens
pub fn tokenize(input: &str) -> Result<Vec<Token>, UnexpectedCharacter> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().enumerate().peekable();
    while let Some((position, character)) = iter.next() {
        match character {
            ',' => tokens.push(Token::LogicalOr),
            '&' => tokens.push(Token::LogicalAnd),
            '!' => tokens.push(Token::LogicalNot),
            '-' => tokens.push(Token::RangeOperator),
            ':' => {
                if let Some(Token::Value(string)) = tokens.pop() {
                    tokens.push(Token::Column(string));
                } else {
                    return Err(UnexpectedCharacter {
                        position,
                        character,
                    });
                }
            }
            ' ' => {}
            '"' => {
                // This impl doesn't support escaping " inside the
                tokens.push(Token::Value(String::from_iter(
                    iter.by_ref()
                        .map(|(_, character)| character)
                        .take_while(|character| *character != '"'),
                )));
            }
            _ => {
                let mut string = String::new();
                string.push(character);
                while let Some((_, character)) = iter.next_if(|(_, character)| {
                    !matches!(*character, ' ' | ',' | '&' | '!' | '-' | ':')
                }) {
                    string.push(character);
                }
                tokens.push(Token::Value(string));
            }
        }
    }
    Ok(tokens)
}

/// Error when the lexer failed at an unexpected character
#[derive(Error, Debug, Copy, Clone)]
#[error("Got unexpected character '{}' at position {}", .character, .position)]
pub struct UnexpectedCharacter {
    /// The character's position (0-indexed, counting unicode codepoints)
    pub position: usize,

    /// The unexpected character
    pub character: char,
}
