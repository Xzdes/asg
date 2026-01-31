//! Ошибки парсера.

use super::token::{Span, Token};
use thiserror::Error;

/// Ошибка парсинга.
#[derive(Error, Debug)]
pub enum ParseError {
    /// Неожиданный токен.
    #[error("Unexpected token at position {}: expected {expected}, found {found}", span.start)]
    UnexpectedToken {
        span: Span,
        expected: String,
        found: String,
    },

    /// Неожиданный конец ввода.
    #[error("Unexpected end of input at position {}: {message}", span.start)]
    UnexpectedEof { span: Span, message: String },

    /// Незакрытая скобка.
    #[error("Unclosed parenthesis at position {}", span.start)]
    UnclosedParen { span: Span },

    /// Неверный литерал.
    #[error("Invalid literal at position {}: {message}", span.start)]
    InvalidLiteral { span: Span, message: String },

    /// Ошибка лексера.
    #[error("Lexer error at position {}: unexpected character", span.start)]
    LexerError { span: Span },

    /// Неизвестная форма.
    #[error("Unknown form '{name}' at position {}", span.start)]
    UnknownForm { span: Span, name: String },

    /// Неверное количество аргументов.
    #[error("Wrong number of arguments for '{name}' at position {}: expected {expected}, got {got}", span.start)]
    WrongArity {
        span: Span,
        name: String,
        expected: String,
        got: usize,
    },
}

impl ParseError {
    /// Создать ошибку "неожиданный токен".
    pub fn unexpected_token(span: Span, expected: impl Into<String>, found: &Token) -> Self {
        Self::UnexpectedToken {
            span,
            expected: expected.into(),
            found: found.to_string(),
        }
    }

    /// Создать ошибку "неожиданный конец".
    pub fn unexpected_eof(span: Span, message: impl Into<String>) -> Self {
        Self::UnexpectedEof {
            span,
            message: message.into(),
        }
    }

    /// Создать ошибку "неизвестная форма".
    pub fn unknown_form(span: Span, name: impl Into<String>) -> Self {
        Self::UnknownForm {
            span,
            name: name.into(),
        }
    }

    /// Создать ошибку "неверное количество аргументов".
    pub fn wrong_arity(
        span: Span,
        name: impl Into<String>,
        expected: impl Into<String>,
        got: usize,
    ) -> Self {
        Self::WrongArity {
            span,
            name: name.into(),
            expected: expected.into(),
            got,
        }
    }

    /// Получить позицию ошибки.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. } => *span,
            Self::UnexpectedEof { span, .. } => *span,
            Self::UnclosedParen { span } => *span,
            Self::InvalidLiteral { span, .. } => *span,
            Self::LexerError { span } => *span,
            Self::UnknownForm { span, .. } => *span,
            Self::WrongArity { span, .. } => *span,
        }
    }
}
