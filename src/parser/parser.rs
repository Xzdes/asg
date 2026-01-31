//! S-Expression парсер для ASG.

use super::error::ParseError;
use super::lexer::Lexer;
use super::token::{Span, Spanned, Token};

/// S-Expression — атом или список.
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    /// Атом — число, строка, идентификатор, символ.
    Atom(Spanned<Atom>),
    /// Список — (expr expr ...)
    List(Spanned<Vec<SExpr>>),
}

/// Атомарное значение.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// Целое число.
    Int(i64),
    /// Число с плавающей точкой.
    Float(f64),
    /// Строка.
    String(String),
    /// Идентификатор (включая ключевые слова).
    Ident(String),
    /// Символ оператора.
    Symbol(String),
}

impl SExpr {
    /// Получить Span выражения.
    pub fn span(&self) -> Span {
        match self {
            SExpr::Atom(spanned) => spanned.span,
            SExpr::List(spanned) => spanned.span,
        }
    }

    /// Проверить, является ли выражение списком.
    pub fn is_list(&self) -> bool {
        matches!(self, SExpr::List(_))
    }

    /// Проверить, является ли выражение атомом.
    pub fn is_atom(&self) -> bool {
        matches!(self, SExpr::Atom(_))
    }

    /// Получить идентификатор из атома.
    pub fn as_ident(&self) -> Option<&str> {
        match self {
            SExpr::Atom(Spanned {
                value: Atom::Ident(s),
                ..
            }) => Some(s),
            _ => None,
        }
    }

    /// Получить символ из атома.
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            SExpr::Atom(Spanned {
                value: Atom::Symbol(s),
                ..
            }) => Some(s),
            _ => None,
        }
    }

    /// Получить целое число из атома.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            SExpr::Atom(Spanned {
                value: Atom::Int(n),
                ..
            }) => Some(*n),
            _ => None,
        }
    }

    /// Получить float из атома.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            SExpr::Atom(Spanned {
                value: Atom::Float(f),
                ..
            }) => Some(*f),
            _ => None,
        }
    }

    /// Получить строку из атома.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SExpr::Atom(Spanned {
                value: Atom::String(s),
                ..
            }) => Some(s),
            _ => None,
        }
    }

    /// Получить список.
    pub fn as_list(&self) -> Option<&[SExpr]> {
        match self {
            SExpr::List(Spanned { value, .. }) => Some(value),
            _ => None,
        }
    }

    /// Получить имя формы (первый элемент списка если это идентификатор или символ).
    pub fn form_name(&self) -> Option<&str> {
        self.as_list()
            .and_then(|list| list.first())
            .and_then(|first| first.as_ident().or_else(|| first.as_symbol()))
    }
}

/// Парсер S-Expression.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// Создать новый парсер.
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    /// Распарсить все S-выражения из исходника.
    pub fn parse_all(&mut self) -> Result<Vec<SExpr>, ParseError> {
        let mut exprs = Vec::new();

        loop {
            let token = self.lexer.peek_token()?;
            if matches!(token.value, Token::Eof) {
                break;
            }
            exprs.push(self.parse_sexpr()?);
        }

        Ok(exprs)
    }

    /// Распарсить одно S-выражение.
    pub fn parse_sexpr(&mut self) -> Result<SExpr, ParseError> {
        let token = self.lexer.next_token()?;

        match token.value {
            Token::LParen => self.parse_list(token.span),
            Token::Int(n) => Ok(SExpr::Atom(Spanned::new(Atom::Int(n), token.span))),
            Token::Float(f) => Ok(SExpr::Atom(Spanned::new(Atom::Float(f), token.span))),
            Token::String(s) => Ok(SExpr::Atom(Spanned::new(Atom::String(s), token.span))),
            Token::Ident(s) => Ok(SExpr::Atom(Spanned::new(Atom::Ident(s), token.span))),
            Token::Symbol(s) => Ok(SExpr::Atom(Spanned::new(Atom::Symbol(s), token.span))),
            Token::RParen => Err(ParseError::unexpected_token(
                token.span,
                "expression",
                &Token::RParen,
            )),
            Token::Eof => Err(ParseError::unexpected_eof(token.span, "expected expression")),
        }
    }

    /// Распарсить список (после открывающей скобки).
    fn parse_list(&mut self, start_span: Span) -> Result<SExpr, ParseError> {
        let mut elements = Vec::new();

        loop {
            let token = self.lexer.peek_token()?;

            match &token.value {
                Token::RParen => {
                    let end_token = self.lexer.next_token()?;
                    let span = start_span.merge(end_token.span);
                    return Ok(SExpr::List(Spanned::new(elements, span)));
                }
                Token::Eof => {
                    return Err(ParseError::UnclosedParen { span: start_span });
                }
                _ => {
                    elements.push(self.parse_sexpr()?);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        let mut parser = Parser::new("42");
        let expr = parser.parse_sexpr().unwrap();
        assert_eq!(expr.as_int(), Some(42));
    }

    #[test]
    fn test_parse_list() {
        let mut parser = Parser::new("(+ 1 2)");
        let expr = parser.parse_sexpr().unwrap();
        assert!(expr.is_list());
        let list = expr.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_symbol(), Some("+"));
        assert_eq!(list[1].as_int(), Some(1));
        assert_eq!(list[2].as_int(), Some(2));
    }

    #[test]
    fn test_parse_nested() {
        let mut parser = Parser::new("(+ (* 2 3) 4)");
        let expr = parser.parse_sexpr().unwrap();
        let list = expr.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list[1].is_list());
    }

    #[test]
    fn test_parse_function() {
        let mut parser = Parser::new("(fn add (a b) (+ a b))");
        let expr = parser.parse_sexpr().unwrap();
        assert_eq!(expr.form_name(), Some("fn"));
    }

    #[test]
    fn test_parse_empty_list() {
        let mut parser = Parser::new("()");
        let expr = parser.parse_sexpr().unwrap();
        assert!(expr.is_list());
        assert_eq!(expr.as_list().unwrap().len(), 0);
    }
}
