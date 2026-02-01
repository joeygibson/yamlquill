//! Error types for YAMLPath parsing and evaluation.

use std::fmt;

/// Errors that can occur during YAMLPath parsing or evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlPathError {
    /// Unexpected token at a specific position.
    UnexpectedToken {
        position: usize,
        found: String,
        expected: String,
    },
    /// Unexpected end of input.
    UnexpectedEnd { expected: String },
    /// Invalid syntax with description.
    InvalidSyntax { message: String },
}

impl fmt::Display for YamlPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlPathError::UnexpectedToken {
                position,
                found,
                expected,
            } => write!(
                f,
                "Unexpected token '{}' at position {}, expected {}",
                found, position, expected
            ),
            YamlPathError::UnexpectedEnd { expected } => {
                write!(f, "Unexpected end of input, expected {}", expected)
            }
            YamlPathError::InvalidSyntax { message } => {
                write!(f, "Invalid YAMLPath syntax: {}", message)
            }
        }
    }
}

impl std::error::Error for YamlPathError {}
