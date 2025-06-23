//! Parse error types for Nix files

use rnix::parser::ParseError as RnixParseError;
use std::fmt;

/// A parse error in a Nix file
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The error message
    pub message: String,
    /// The position in the file where the error occurred
    pub position: Option<Position>,
    /// The kind of error
    pub kind: ParseErrorKind,
}

impl ParseError {
    /// Create a parse error from an rnix parse error
    pub fn from_rnix(error: RnixParseError) -> Self {
        Self {
            message: error.to_string(),
            position: None, // rnix doesn't provide position info directly
            kind: ParseErrorKind::SyntaxError,
        }
    }

    /// Create a new parse error
    pub fn new(message: impl Into<String>, kind: ParseErrorKind) -> Self {
        Self {
            message: message.into(),
            position: None,
            kind,
        }
    }

    /// Set the position of the error
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pos) = &self.position {
            write!(f, "{}:{}: {}", pos.line, pos.column, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

/// The kind of parse error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// A syntax error in the Nix code
    SyntaxError,
    /// An invalid flake structure
    InvalidFlake,
    /// An invalid module structure
    InvalidModule,
    /// An invalid attribute
    InvalidAttribute,
    /// A missing required field
    MissingField,
    /// An unexpected token
    UnexpectedToken,
    /// An unclosed delimiter
    UnclosedDelimiter,
    /// An invalid string escape
    InvalidEscape,
}

/// A position in a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The line number (1-indexed)
    pub line: usize,
    /// The column number (1-indexed)
    pub column: usize,
    /// The byte offset in the file
    pub offset: usize,
} 