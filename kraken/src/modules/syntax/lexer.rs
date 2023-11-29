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
    let mut current_string = String::new();
    for (position, character) in input.chars().enumerate() {
        if matches!(character, ' ' | ',' | '&' | '!' | '-' | ':') && current_string.len() > 0 {
            tokens.push(Token::Value(current_string.to_string()));
            current_string.clear();
        }

        match character {
            ',' => tokens.push(Token::LogicalOr),
            '&' => tokens.push(Token::LogicalAnd),
            '!' => tokens.push(Token::LogicalNot),
            '-' => tokens.push(Token::RangeOperator),
            ':' => {
                let Some(Token::Value(string)) = tokens.pop() else {
                    return Err(UnexpectedCharacter {
                        position,
                        character,
                    });
                };
                tokens.push(Token::Column(string));
            }
            ' ' => {}
            _ => current_string.push(character),
        }
    }
    if current_string.len() > 0 {
        tokens.push(Token::Value(current_string.trim().to_string()));
    }
    Ok(tokens)
}

#[derive(Error, Debug, Copy, Clone)]
#[error("Got unexpected character '{}' at position {}", .character, .position)]
pub struct UnexpectedCharacter {
    pub position: usize,
    pub character: char,
}
