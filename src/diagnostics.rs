//! Error types that reference `.sn` source locations, per the TinyS design goal
//! of never forcing users to debug generated Rust line numbers.

use std::fmt;

/// A location in a `.sn` source file (1-based line and column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col }
    }
}

/// A compiler error carrying enough context to point back at TinyS source.
#[derive(Debug, Clone)]
pub struct TinysError {
    pub stage: Stage,
    pub message: String,
    pub file: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Lex,
    Parse,
}

impl Stage {
    fn label(self) -> &'static str {
        match self {
            Stage::Lex => "lex error",
            Stage::Parse => "parse error",
        }
    }
}

impl TinysError {
    pub fn new(stage: Stage, message: impl Into<String>, file: &str, span: Span) -> Self {
        TinysError {
            stage,
            message: message.into(),
            file: file.to_string(),
            span,
        }
    }
}

impl fmt::Display for TinysError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error: {}: {}\n  --> {}:{}:{}",
            self.stage.label(),
            self.message,
            self.file,
            self.span.line,
            self.span.col,
        )
    }
}

impl std::error::Error for TinysError {}
