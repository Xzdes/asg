//! Токены и позиции для S-Expression парсера.

use serde::{Deserialize, Serialize};

/// Позиция в исходном коде.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// Начальная позиция (байт).
    pub start: usize,
    /// Конечная позиция (байт).
    pub end: usize,
}

impl Span {
    /// Создать новый Span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Объединить два Span.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Токен с позицией.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Типы токенов для S-Expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Открывающая скобка `(`
    LParen,
    /// Закрывающая скобка `)`
    RParen,

    /// Целое число
    Int(i64),
    /// Число с плавающей точкой
    Float(f64),
    /// Строковый литерал
    String(String),

    /// Идентификатор или ключевое слово
    Ident(String),
    /// Символ оператора (+, -, *, /, etc.)
    Symbol(String),

    /// Конец файла
    Eof,
}

impl Token {
    /// Проверить, является ли токен открывающей скобкой.
    pub fn is_lparen(&self) -> bool {
        matches!(self, Token::LParen)
    }

    /// Проверить, является ли токен закрывающей скобкой.
    pub fn is_rparen(&self) -> bool {
        matches!(self, Token::RParen)
    }

    /// Проверить, является ли токен атомом (не скобкой).
    pub fn is_atom(&self) -> bool {
        matches!(
            self,
            Token::Int(_) | Token::Float(_) | Token::String(_) | Token::Ident(_) | Token::Symbol(_)
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Symbol(s) => write!(f, "{}", s),
            Token::Eof => write!(f, "EOF"),
        }
    }
}
