//! Go to definition implementation.

use tower_lsp::lsp_types::*;
use asg_lang::asg::ASG;
use asg_lang::nodecodes::NodeType;

/// Информация о определении символа.
#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    /// Имя символа
    pub name: String,
    /// Начальная позиция в исходном коде
    pub start_offset: usize,
    /// Конечная позиция в исходном коде
    pub end_offset: usize,
}

/// Найти определение символа под курсором.
pub fn find_definition(
    content: &str,
    position: Position,
    asg: Option<&ASG>,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    // Получаем слово под курсором
    let word = get_word_at_position(content, position)?;

    // Ищем определение в ASG
    let asg = asg?;
    let def = find_definition_in_asg(asg, &word)?;

    // Конвертируем в Location
    let range = Range {
        start: offset_to_position(content, def.start_offset),
        end: offset_to_position(content, def.end_offset),
    };

    Some(GotoDefinitionResponse::Scalar(Location {
        uri: uri.clone(),
        range,
    }))
}

/// Получить слово под позицией курсора.
fn get_word_at_position(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    // Находим границы слова
    let chars: Vec<char> = line.chars().collect();

    // Ищем начало слова
    let mut start = col;
    while start > 0 && is_identifier_char(chars.get(start - 1).copied().unwrap_or(' ')) {
        start -= 1;
    }

    // Ищем конец слова
    let mut end = col;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    if start >= end {
        return None;
    }

    let word: String = chars[start..end].iter().collect();
    Some(word)
}

/// Проверка символа идентификатора.
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '!'
}

/// Найти определение символа в ASG.
fn find_definition_in_asg(asg: &ASG, name: &str) -> Option<DefinitionInfo> {
    for node in &asg.nodes {
        match node.node_type {
            NodeType::Function | NodeType::Variable => {
                if let Some(node_name) = node.get_name() {
                    if node_name == name {
                        if let Some(span) = node.span {
                            return Some(DefinitionInfo {
                                name: node_name,
                                start_offset: span.start,
                                end_offset: span.end,
                            });
                        }
                    }
                }
            }
            _ => continue,
        }
    }
    None
}

/// Найти все ссылки на символ.
pub fn find_references(
    content: &str,
    position: Position,
    asg: Option<&ASG>,
    uri: &Url,
) -> Option<Vec<Location>> {
    let word = get_word_at_position(content, position)?;
    let asg = asg?;

    let mut locations = Vec::new();

    // Ищем все узлы VarRef с этим именем
    for node in &asg.nodes {
        if node.node_type == NodeType::VarRef {
            if let Some(node_name) = node.get_name() {
                if node_name == word {
                    if let Some(span) = node.span {
                        let range = Range {
                            start: offset_to_position(content, span.start),
                            end: offset_to_position(content, span.end),
                        };
                        locations.push(Location {
                            uri: uri.clone(),
                            range,
                        });
                    }
                }
            }
        }
    }

    // Также добавляем определение
    if let Some(def) = find_definition_in_asg(asg, &word) {
        let range = Range {
            start: offset_to_position(content, def.start_offset),
            end: offset_to_position(content, def.end_offset),
        };
        locations.insert(0, Location {
            uri: uri.clone(),
            range,
        });
    }

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

/// Конвертировать offset в Position.
fn offset_to_position(content: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in content.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position { line, character: col }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_word_at_position() {
        let content = "(fn hello (x) (+ x 1))";

        // Курсор на "hello"
        let word = get_word_at_position(content, Position { line: 0, character: 4 });
        assert_eq!(word, Some("hello".to_string()));

        // Курсор на "fn"
        let word = get_word_at_position(content, Position { line: 0, character: 1 });
        assert_eq!(word, Some("fn".to_string()));
    }

    #[test]
    fn test_is_identifier_char() {
        assert!(is_identifier_char('a'));
        assert!(is_identifier_char('Z'));
        assert!(is_identifier_char('0'));
        assert!(is_identifier_char('-'));
        assert!(is_identifier_char('_'));
        assert!(is_identifier_char('?'));
        assert!(!is_identifier_char(' '));
        assert!(!is_identifier_char('('));
    }
}
