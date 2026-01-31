//! Hover информация для LSP.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};
use asg_lang::asg::ASG;
use asg_lang::nodecodes::NodeType;

/// Получить hover информацию.
pub fn get_hover_info(
    content: &str,
    position: Position,
    asg: Option<&ASG>,
) -> Option<Hover> {
    let word = get_word_at_position(content, position)?;

    // === Ключевые слова ===
    let keyword_info = get_keyword_info(&word);
    if let Some(info) = keyword_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: None,
        });
    }

    // === Встроенные функции ===
    let builtin_info = get_builtin_info(&word);
    if let Some(info) = builtin_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: None,
        });
    }

    // === Символы из ASG ===
    if let Some(asg) = asg {
        for node in &asg.nodes {
            let name = node.get_name();
            if name.as_ref() == Some(&word) {
                let info = get_node_info(node);
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: None,
                });
            }
        }
    }

    None
}

/// Получить информацию о ключевом слове.
fn get_keyword_info(word: &str) -> Option<String> {
    let info = match word {
        "fn" => "**fn** - Function definition\n\n```asg\n(fn name (arg1 arg2) body)\n```",
        "let" => "**let** - Variable binding\n\n```asg\n(let name value)\n```",
        "if" => "**if** - Conditional expression\n\n```asg\n(if condition then-branch else-branch)\n```",
        "do" => "**do** - Block of expressions\n\n```asg\n(do expr1 expr2 ... result)\n```",
        "loop" => "**loop** - Loop construct\n\n```asg\n(loop condition body)\n```",
        "for" => "**for** - For loop\n\n```asg\n(for var iterable body)\n```",
        "match" => "**match** - Pattern matching\n\n```asg\n(match value\n  (pattern1 result1)\n  (pattern2 result2))\n```",
        "lambda" => "**lambda** - Anonymous function\n\n```asg\n(lambda (args) body)\n```",
        "import" => "**import** - Import module\n\n```asg\n(import \"module-name\")\n(import \"module\" :as alias)\n(import \"module\" :only (name1 name2))\n```",
        "export" => "**export** - Export from module\n\n```asg\n(export name1 name2 ...)\n```",
        "module" => "**module** - Module definition\n\n```asg\n(module name\n  (export ...)\n  definitions...)\n```",
        "try" => "**try** - Try-catch block\n\n```asg\n(try expr (catch e handler))\n```",
        "throw" => "**throw** - Throw error\n\n```asg\n(throw \"error message\")\n```",
        _ => return None,
    };
    Some(info.to_string())
}

/// Получить информацию о встроенной функции.
fn get_builtin_info(word: &str) -> Option<String> {
    let info = match word {
        // Арифметика
        "+" => "**+** - Addition\n\n```asg\n(+ a b)\n```\nReturns a + b",
        "-" => "**-** - Subtraction\n\n```asg\n(- a b)\n```\nReturns a - b",
        "*" => r#"__*__ - Multiplication

```asg
(* a b)
```
Returns a * b"#,
        "/" => r#"__/__ - Division

```asg
(/ a b)
```
Returns a / b"#,
        "%" => r#"__%__ - Modulo

```asg
(% a b)
```
Returns a % b"#,

        // Сравнение
        "=" => "**=** - Equality\n\n```asg\n(= a b)\n```\nReturns `true` if `a == b`",
        "!=" => "**!=** - Inequality\n\n```asg\n(!= a b)\n```\nReturns `true` if `a != b`",
        "<" => "**<** - Less than\n\n```asg\n(< a b)\n```\nReturns `true` if `a < b`",
        ">" => "**>** - Greater than\n\n```asg\n(> a b)\n```\nReturns `true` if `a > b`",

        // Математика
        "sqrt" => "**sqrt** - Square root\n\n```asg\n(sqrt x)\n```\nReturns √x",
        "sin" => "**sin** - Sine\n\n```asg\n(sin x)\n```\nReturns sin(x)",
        "cos" => "**cos** - Cosine\n\n```asg\n(cos x)\n```\nReturns cos(x)",
        "pow" => "**pow** - Power\n\n```asg\n(pow base exp)\n```\nReturns base^exp",
        "abs" => "**abs** - Absolute value\n\n```asg\n(abs x)\n```\nReturns |x|",
        "PI" => "**PI** - Pi constant\n\nValue: 3.14159265358979...",
        "E" => "**E** - Euler's number\n\nValue: 2.71828182845904...",

        // Ввод/вывод
        "print" => "**print** - Print value\n\n```asg\n(print value)\n```\nPrints value to stdout",
        "input" => "**input** - Read input\n\n```asg\n(input)\n(input \"prompt\")\n```\nReads line from stdin",

        // Массивы
        "map" => "**map** - Map over array\n\n```asg\n(map array fn)\n```\nApplies fn to each element",
        "filter" => "**filter** - Filter array\n\n```asg\n(filter array predicate)\n```\nKeeps elements where predicate is true",
        "reduce" => "**reduce** - Reduce array\n\n```asg\n(reduce array init fn)\n```\nFolds array with fn",
        "range" => "**range** - Create range\n\n```asg\n(range start end)\n(range start end step)\n```\nCreates array [start, start+step, ..., end)",

        // Строки
        "str-concat" | "concat" => "**concat** - Concatenate strings\n\n```asg\n(concat s1 s2)\n```\nReturns s1 + s2",
        "str-length" => "**str-length** - String length\n\n```asg\n(str-length s)\n```\nReturns length of s",

        _ => return None,
    };
    Some(info.to_string())
}

/// Получить информацию об узле ASG.
fn get_node_info(node: &asg_lang::asg::Node) -> String {
    let name = node.get_name().unwrap_or_else(|| "<anonymous>".to_string());

    match node.node_type {
        NodeType::Function => {
            let params: Vec<_> = node
                .edges
                .iter()
                .filter(|e| e.edge_type == asg_lang::nodecodes::EdgeType::FunctionParameter)
                .filter_map(|_| Some("arg".to_string()))
                .collect();

            format!(
                "**function** `{}`\n\n```asg\n(fn {} ({}) ...)\n```",
                name,
                name,
                params.join(" ")
            )
        }
        NodeType::Variable => {
            format!("**variable** `{}`", name)
        }
        NodeType::Module => {
            format!("**module** `{}`", name)
        }
        _ => {
            format!("{:?} `{}`", node.node_type, name)
        }
    }
}

/// Получить слово в позиции курсора.
fn get_word_at_position(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let line_idx = position.line as usize;

    if line_idx >= lines.len() {
        return None;
    }

    let line = lines[line_idx];
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

    // Находим границы слова
    let chars: Vec<char> = line.chars().collect();

    let is_word_char = |c: char| !c.is_whitespace() && c != '(' && c != ')' && c != '[' && c != ']';

    // Ищем начало слова
    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    // Ищем конец слова
    let mut end = col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    Some(chars[start..end].iter().collect())
}
