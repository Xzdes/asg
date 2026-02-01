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

/// Calculate line and column from byte offset.
pub fn calculate_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let prefix = &source[..byte_offset.min(source.len())];
    let line = prefix.matches('\n').count() + 1;
    let col = match prefix.rfind('\n') {
        Some(i) => byte_offset - i,
        None => byte_offset + 1,
    };
    (line, col)
}

/// Known forms for suggestions.
const KNOWN_FORMS: &[&str] = &[
    // Arithmetic
    "+", "-", "*", "/", "//", "%", "neg",
    // Comparison
    "==", "!=", "<", "<=", ">", ">=",
    // Logic
    "and", "or", "not", "&&", "||", "!",
    // Variables
    "let", "set",
    // Control
    "if", "do", "while", "loop", "for", "break", "continue", "return", "match",
    // Functions
    "fn", "lambda",
    // Data
    "array", "index", "nth", "first", "second", "third", "last", "length",
    "map", "filter", "reduce", "dict", "record", "field",
    // I/O
    "print", "input", "read-file", "write-file",
    // Strings
    "concat", "str-length", "substring", "str-split", "str-join",
    // Math
    "sqrt", "sin", "cos", "tan", "pow", "abs", "floor", "ceil", "round", "min", "max",
    // Error handling
    "try", "throw", "is-error", "error-message",
    // Pipeline
    "|>", "pipe", "compose",
    // Lazy
    "iterate", "repeat", "cycle", "take-lazy", "collect",
    // Modules
    "module", "import", "export",
];

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

    /// Format error with source context showing line:column.
    pub fn format_with_source(&self, source: &str) -> String {
        let span = self.span();
        let (line, col) = calculate_line_col(source, span.start);

        // Get the line content
        let lines: Vec<&str> = source.lines().collect();
        let line_content = lines.get(line.saturating_sub(1)).unwrap_or(&"");

        // Build error message
        let mut msg = format!("Error at line {}, column {}:\n", line, col);
        msg.push_str(&format!("  {}\n", line_content));

        // Add caret pointing to error position
        let caret_pos = col.saturating_sub(1);
        msg.push_str(&format!("  {}^\n", " ".repeat(caret_pos)));

        // Add error description
        msg.push_str(&format!("{}", self));

        // Add suggestion if available
        if let Some(suggestion) = self.suggest() {
            msg.push_str(&format!("\n\nHint: {}", suggestion));
        }

        msg
    }

    /// Get suggestion for fixing the error.
    pub fn suggest(&self) -> Option<String> {
        match self {
            Self::UnknownForm { name, .. } => {
                // Find similar form names
                if let Some(similar) = find_similar_form(name) {
                    Some(format!("Did you mean '{}'?", similar))
                } else {
                    Some("Check the documentation for available forms: docs/BUILTIN_FUNCTIONS.md".to_string())
                }
            }
            Self::WrongArity { name, expected, got, .. } => {
                Some(format!(
                    "'{}' requires {} argument(s), but {} provided. Check syntax: ({} arg1 arg2 ...)",
                    name, expected, got, name
                ))
            }
            Self::UnclosedParen { .. } => {
                Some("Make sure all opening parentheses '(' have matching closing ')'".to_string())
            }
            Self::UnexpectedEof { .. } => {
                Some("The expression is incomplete. Check for missing closing parentheses or arguments.".to_string())
            }
            Self::UnexpectedToken { expected, found, .. } => {
                if found == ")" && expected.contains("expression") {
                    Some("Empty list '()' is valid. For function calls, provide arguments: (fn arg1 arg2)".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Find similar form name for "did you mean" suggestions.
fn find_similar_form(name: &str) -> Option<&'static str> {
    let name_lower = name.to_lowercase();

    // Check for exact case-insensitive match
    for &form in KNOWN_FORMS {
        if form.to_lowercase() == name_lower {
            return Some(form);
        }
    }

    // Check for common typos and close matches
    for &form in KNOWN_FORMS {
        // Check if one is prefix of another
        if form.starts_with(&name_lower) || name_lower.starts_with(form) {
            return Some(form);
        }

        // Simple Levenshtein-like check: allow 1 character difference for short names
        if name.len() <= 5 && form.len() <= 5 {
            let diff = levenshtein_distance(name, form);
            if diff <= 1 {
                return Some(form);
            }
        } else if name.len() > 5 && form.len() > 5 {
            let diff = levenshtein_distance(name, form);
            if diff <= 2 {
                return Some(form);
            }
        }
    }

    None
}

/// Simple Levenshtein distance for short strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }

    dp[m][n]
}
