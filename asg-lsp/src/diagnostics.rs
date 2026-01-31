//! Диагностика для LSP.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Получить диагностику из сообщения об ошибке.
pub fn get_diagnostics(error_message: &str, content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Пытаемся извлечь позицию из сообщения об ошибке
    // Формат: "at line X, column Y" или "at position N"
    let (line, col) = extract_position(error_message, content);

    diagnostics.push(Diagnostic {
        range: Range {
            start: Position {
                line: line as u32,
                character: col as u32,
            },
            end: Position {
                line: line as u32,
                character: (col + 1) as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("asg".to_string()),
        message: clean_error_message(error_message),
        related_information: None,
        tags: None,
        data: None,
    });

    diagnostics
}

/// Извлечь позицию из сообщения об ошибке.
fn extract_position(error_message: &str, _content: &str) -> (usize, usize) {
    // Ищем паттерны типа "line 5", "column 10", "position 42"
    let lower = error_message.to_lowercase();

    // Паттерн "at line X"
    if let Some(idx) = lower.find("line ") {
        let rest = &lower[idx + 5..];
        if let Some(num) = parse_leading_number(rest) {
            // Ищем column
            if let Some(col_idx) = rest.find("column ") {
                let col_rest = &rest[col_idx + 7..];
                if let Some(col) = parse_leading_number(col_rest) {
                    return (num.saturating_sub(1), col.saturating_sub(1));
                }
            }
            return (num.saturating_sub(1), 0);
        }
    }

    // Паттерн "at position X"
    if let Some(idx) = lower.find("position ") {
        let rest = &lower[idx + 9..];
        if let Some(pos) = parse_leading_number(rest) {
            // Конвертируем offset в line:col
            return (0, pos);
        }
    }

    // По умолчанию — начало файла
    (0, 0)
}

/// Парсить число в начале строки.
fn parse_leading_number(s: &str) -> Option<usize> {
    let num_str: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

/// Очистить сообщение об ошибке от технических деталей.
fn clean_error_message(msg: &str) -> String {
    // Убираем префиксы типа "Parse error: "
    let msg = msg
        .strip_prefix("Parse error: ")
        .or_else(|| msg.strip_prefix("Error: "))
        .unwrap_or(msg);

    msg.to_string()
}
