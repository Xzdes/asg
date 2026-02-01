//! Лексер для S-Expression синтаксиса ASG.

use logos::Logos;

use super::error::ParseError;
use super::token::{Span, Spanned, Token};

/// Внутренние токены для logos.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")] // Пропускаем пробелы
#[logos(skip r";[^\n]*")] // Пропускаем комментарии ; до конца строки
enum LogosToken {
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    // Булевы литералы (до идентификаторов!)
    #[token("true")]
    True,

    #[token("false")]
    False,

    // Float (должен быть до Int для правильного приоритета)
    #[regex(r"-?[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    // Integer
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    // Hex integer
    #[regex(r"0[xX][0-9a-fA-F]+", |lex| i64::from_str_radix(&lex.slice()[2..], 16).ok())]
    HexInt(i64),

    // Binary integer
    #[regex(r"0[bB][01]+", |lex| i64::from_str_radix(&lex.slice()[2..], 2).ok())]
    BinInt(i64),

    // Строковый литерал
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Убираем кавычки и обрабатываем escape-последовательности
        Some(unescape_string(&s[1..s.len()-1]))
    })]
    String(String),

    // Символьные операторы (многосимвольные сначала!)
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("|>")]
    Pipe,

    // Многосимвольные операторы
    #[token("//")]
    DoubleSlash,

    // Односимвольные операторы
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("!")]
    Bang,
    #[token(":")]
    Colon,

    // Идентификатор (включая ключевые слова с дефисом: tensor-add)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_-]*", |lex| lex.slice().to_string())]
    Ident(String),
}

/// Обработка escape-последовательностей в строке.
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Лексер для ASG S-Expression.
pub struct Lexer<'a> {
    logos: logos::Lexer<'a, LogosToken>,
    source: &'a str,
    peeked: Option<Spanned<Token>>,
}

impl<'a> Lexer<'a> {
    /// Создать новый лексер.
    pub fn new(source: &'a str) -> Self {
        Self {
            logos: LogosToken::lexer(source),
            source,
            peeked: None,
        }
    }

    /// Получить следующий токен.
    pub fn next_token(&mut self) -> Result<Spanned<Token>, ParseError> {
        if let Some(token) = self.peeked.take() {
            return Ok(token);
        }

        self.read_token()
    }

    /// Посмотреть на следующий токен без его потребления.
    pub fn peek_token(&mut self) -> Result<&Spanned<Token>, ParseError> {
        if self.peeked.is_none() {
            self.peeked = Some(self.read_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    /// Прочитать токен из logos.
    fn read_token(&mut self) -> Result<Spanned<Token>, ParseError> {
        match self.logos.next() {
            Some(Ok(logos_token)) => {
                let span = Span::new(self.logos.span().start, self.logos.span().end);
                let token = self.convert_token(logos_token);
                Ok(Spanned::new(token, span))
            }
            Some(Err(())) => {
                let span = Span::new(self.logos.span().start, self.logos.span().end);
                Err(ParseError::LexerError { span })
            }
            None => {
                let pos = self.source.len();
                Ok(Spanned::new(Token::Eof, Span::new(pos, pos)))
            }
        }
    }

    /// Конвертировать внутренний токен logos в публичный Token.
    fn convert_token(&self, logos_token: LogosToken) -> Token {
        match logos_token {
            LogosToken::LParen => Token::LParen,
            LogosToken::RParen => Token::RParen,
            LogosToken::True => Token::Ident("true".to_string()),
            LogosToken::False => Token::Ident("false".to_string()),
            LogosToken::Int(n) => Token::Int(n),
            LogosToken::HexInt(n) => Token::Int(n),
            LogosToken::BinInt(n) => Token::Int(n),
            LogosToken::Float(f) => Token::Float(f),
            LogosToken::String(s) => Token::String(s),
            LogosToken::Ident(s) => Token::Ident(s),
            // Операторы
            LogosToken::Plus => Token::Symbol("+".to_string()),
            LogosToken::Minus => Token::Symbol("-".to_string()),
            LogosToken::Star => Token::Symbol("*".to_string()),
            LogosToken::Slash => Token::Symbol("/".to_string()),
            LogosToken::DoubleSlash => Token::Symbol("//".to_string()),
            LogosToken::Percent => Token::Symbol("%".to_string()),
            LogosToken::Lt => Token::Symbol("<".to_string()),
            LogosToken::Gt => Token::Symbol(">".to_string()),
            LogosToken::Le => Token::Symbol("<=".to_string()),
            LogosToken::Ge => Token::Symbol(">=".to_string()),
            LogosToken::Eq => Token::Symbol("==".to_string()),
            LogosToken::Ne => Token::Symbol("!=".to_string()),
            LogosToken::And => Token::Symbol("&&".to_string()),
            LogosToken::Or => Token::Symbol("||".to_string()),
            LogosToken::Pipe => Token::Symbol("|>".to_string()),
            LogosToken::Bang => Token::Symbol("!".to_string()),
            LogosToken::Colon => Token::Symbol(":".to_string()),
        }
    }

    /// Получить текущую позицию.
    pub fn position(&self) -> usize {
        self.logos.span().start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let mut lexer = Lexer::new("(+ 1 2)");

        assert!(matches!(lexer.next_token().unwrap().value, Token::LParen));
        assert!(matches!(
            lexer.next_token().unwrap().value,
            Token::Symbol(s) if s == "+"
        ));
        assert!(matches!(lexer.next_token().unwrap().value, Token::Int(1)));
        assert!(matches!(lexer.next_token().unwrap().value, Token::Int(2)));
        assert!(matches!(lexer.next_token().unwrap().value, Token::RParen));
        assert!(matches!(lexer.next_token().unwrap().value, Token::Eof));
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        match lexer.next_token().unwrap().value {
            Token::String(s) => assert_eq!(s, "hello\nworld"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_lexer_comments() {
        let mut lexer = Lexer::new("; comment\n42");
        assert!(matches!(lexer.next_token().unwrap().value, Token::Int(42)));
    }

    #[test]
    fn test_lexer_float() {
        let mut lexer = Lexer::new("3.14");
        match lexer.next_token().unwrap().value {
            Token::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }
}
