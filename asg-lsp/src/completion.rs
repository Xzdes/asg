//! Автодополнение для LSP.

use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};
use asg_lang::asg::ASG;
use asg_lang::nodecodes::NodeType;

/// Получить элементы автодополнения.
pub fn get_completions(
    content: &str,
    position: Position,
    asg: Option<&ASG>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Получаем контекст (что набирает пользователь)
    let prefix = get_word_at_position(content, position);

    // === Ключевые слова языка ===
    let keywords = [
        ("fn", "Function definition"),
        ("let", "Variable binding"),
        ("if", "Conditional expression"),
        ("do", "Block of expressions"),
        ("loop", "Loop construct"),
        ("match", "Pattern matching"),
        ("import", "Import module"),
        ("export", "Export from module"),
        ("module", "Module definition"),
        ("lambda", "Anonymous function"),
        ("for", "For loop"),
        ("try", "Try-catch block"),
        ("throw", "Throw error"),
    ];

    for (kw, doc) in keywords {
        if kw.starts_with(&prefix) || prefix.is_empty() {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(doc.to_string()),
                insert_text: Some(kw.to_string()),
                ..Default::default()
            });
        }
    }

    // === Встроенные функции ===
    let builtins = [
        // Арифметика
        ("+", "Addition"),
        ("-", "Subtraction"),
        ("*", "Multiplication"),
        ("/", "Division"),
        ("%", "Modulo"),
        // Сравнение
        ("=", "Equality"),
        ("!=", "Inequality"),
        ("<", "Less than"),
        (">", "Greater than"),
        ("<=", "Less or equal"),
        (">=", "Greater or equal"),
        // Логика
        ("and", "Logical AND"),
        ("or", "Logical OR"),
        ("not", "Logical NOT"),
        // Ввод/вывод
        ("print", "Print value"),
        ("input", "Read input"),
        // Математика
        ("sqrt", "Square root"),
        ("sin", "Sine"),
        ("cos", "Cosine"),
        ("tan", "Tangent"),
        ("exp", "Exponential"),
        ("ln", "Natural logarithm"),
        ("abs", "Absolute value"),
        ("floor", "Floor"),
        ("ceil", "Ceiling"),
        ("round", "Round"),
        ("min", "Minimum"),
        ("max", "Maximum"),
        ("pow", "Power"),
        ("PI", "Pi constant"),
        ("E", "Euler's number"),
        // Массивы
        ("array", "Create array"),
        ("get", "Get element"),
        ("length", "Array length"),
        ("map", "Map over array"),
        ("filter", "Filter array"),
        ("reduce", "Reduce array"),
        ("range", "Create range"),
        ("append", "Append element"),
        ("concat", "Concatenate arrays"),
        // Строки
        ("str-length", "String length"),
        ("str-concat", "Concatenate strings"),
        ("str-split", "Split string"),
        ("str-join", "Join strings"),
        ("to-string", "Convert to string"),
        // Словари
        ("dict", "Create dictionary"),
        ("dict-get", "Get from dictionary"),
        ("dict-set", "Set in dictionary"),
        ("dict-keys", "Get dictionary keys"),
    ];

    for (name, doc) in builtins {
        if name.starts_with(&prefix) || prefix.is_empty() {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(doc.to_string()),
                insert_text: Some(name.to_string()),
                ..Default::default()
            });
        }
    }

    // === Символы из ASG ===
    if let Some(asg) = asg {
        for node in &asg.nodes {
            let (name, kind) = match node.node_type {
                NodeType::Function => {
                    let name = node.get_name();
                    (name, CompletionItemKind::FUNCTION)
                }
                NodeType::Variable => {
                    let name = node.get_name();
                    (name, CompletionItemKind::VARIABLE)
                }
                _ => continue,
            };

            if let Some(name) = name {
                if name.starts_with(&prefix) || prefix.is_empty() {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(kind),
                        detail: Some(format!("{:?}", node.node_type)),
                        insert_text: Some(name),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // === Сниппеты ===
    let snippets = [
        ("fn-def", "(fn ${1:name} (${2:args}) ${3:body})", "Function definition"),
        ("let-def", "(let ${1:name} ${2:value})", "Variable binding"),
        ("if-else", "(if ${1:cond} ${2:then} ${3:else})", "If expression"),
        ("for-loop", "(for ${1:var} ${2:iterable} ${3:body})", "For loop"),
        ("match-expr", "(match ${1:value}\n  (${2:pattern} ${3:result}))", "Match expression"),
    ];

    for (trigger, snippet, doc) in snippets {
        if trigger.starts_with(&prefix) || prefix.is_empty() {
            items.push(CompletionItem {
                label: trigger.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(doc.to_string()),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
    }

    items
}

/// Получить слово в позиции курсора.
fn get_word_at_position(content: &str, position: Position) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return String::new();
    }

    let line = lines[line_idx];
    let col = position.character as usize;

    if col > line.len() {
        return String::new();
    }

    // Находим начало слова
    let before = &line[..col];
    let start = before
        .rfind(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '[' || c == ']')
        .map(|i| i + 1)
        .unwrap_or(0);

    before[start..].to_string()
}
