//! ASG CLI - интерактивная оболочка и интерпретатор.
//!
//! Использование:
//!   asg              - запустить REPL
//!   asg <file.asg>   - выполнить файл
//!   asg -e "expr"    - выполнить выражение
//!   asg --help       - справка

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::fs;
use std::process;

use asg_lang::interpreter::{Interpreter, Value};
use asg_lang::parser::{parse, parse_expr};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = r#"
ASG - AI-friendly programming language

USAGE:
    asg                  Start REPL (interactive mode)
    asg <file.asg>       Execute a ASG file
    asg -e "<expr>"      Evaluate an expression
    asg --help, -h       Show this help
    asg --version, -v    Show version

REPL COMMANDS:
    :help, :h                Show help
    :quit, :q, :exit         Exit REPL
    :clear, :c               Clear screen
    :reset, :r               Reset interpreter state
    :load <file>             Load and execute a file
    :env                     Show defined variables
    :funcs                   Show defined functions
    :ast <expr>              Show ASG for expression
    :type <expr>             Show inferred type

EXAMPLES:
    asg -e "(+ 1 2)"
    asg examples/demo.asg
    asg

SYNTAX (S-Expression):
    (+ 1 2)                  ; Addition -> 3
    (* (+ 2 3) 4)            ; Nested -> 20
    (let x 42)               ; Variable declaration
    (if (< x 0) (neg x) x)   ; Conditional
    (fn square (x) (* x x))  ; Function definition
    (square 5)               ; Function call -> 25
    (|> 10 square (* 2))     ; Pipe operator -> 200
    (dict "a" 1 "b" 2)       ; Dictionary
    (input "Name: ")         ; User input
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_repl(),
        2 => match args[1].as_str() {
            "--help" | "-h" => {
                println!("{}", HELP);
            }
            "--version" | "-v" => {
                println!("ASG {}", VERSION);
            }
            file => run_file(file),
        },
        3 => {
            if args[1] == "-e" || args[1] == "--eval" {
                run_expr(&args[2]);
            } else {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Use --help for usage information.");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Too many arguments.");
            eprintln!("Use --help for usage information.");
            process::exit(1);
        }
    }
}

/// Запустить REPL.
fn run_repl() {
    println!("ASG {} - AI-friendly language", VERSION);
    println!("Type :help for commands, :quit to exit.\n");

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize readline: {}", e);
            process::exit(1);
        }
    };

    let mut interpreter = Interpreter::new();
    let history_path = dirs_next::data_dir()
        .map(|p| p.join("asg").join("history.txt"))
        .unwrap_or_else(|| std::path::PathBuf::from(".asgapse_history"));

    // Загрузить историю
    let _ = rl.load_history(&history_path);

    loop {
        let readline = rl.readline("asg> ");

        match readline {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Добавить в историю
                let _ = rl.add_history_entry(line);

                // Обработать команды REPL
                if line.starts_with(':') {
                    match handle_command(line, &mut interpreter) {
                        CommandResult::Continue => continue,
                        CommandResult::Exit => break,
                        CommandResult::Reset => {
                            interpreter = Interpreter::new();
                            println!("Interpreter state reset.");
                            continue;
                        }
                    }
                }

                // Выполнить код
                execute_line(line, &mut interpreter);
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Сохранить историю
    if let Some(parent) = history_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = rl.save_history(&history_path);
}

enum CommandResult {
    Continue,
    Exit,
    Reset,
}

fn handle_command(cmd: &str, interpreter: &mut Interpreter) -> CommandResult {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts[0];
    let arg = parts.get(1).map(|s| s.trim());

    match command {
        ":help" | ":h" => {
            println!("{}", HELP);
            CommandResult::Continue
        }
        ":quit" | ":q" | ":exit" => CommandResult::Exit,
        ":clear" | ":c" => {
            print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
            CommandResult::Continue
        }
        ":reset" | ":r" => CommandResult::Reset,
        ":load" | ":l" => {
            if let Some(path) = arg {
                load_file(path, interpreter);
            } else {
                println!("Usage: :load <file.asg>");
            }
            CommandResult::Continue
        }
        ":env" | ":vars" => {
            show_env(interpreter);
            CommandResult::Continue
        }
        ":funcs" | ":functions" => {
            show_functions(interpreter);
            CommandResult::Continue
        }
        ":ast" => {
            if let Some(expr) = arg {
                show_ast(expr);
            } else {
                println!("Usage: :ast <expression>");
            }
            CommandResult::Continue
        }
        ":type" | ":t" => {
            if let Some(expr) = arg {
                show_type(expr, interpreter);
            } else {
                println!("Usage: :type <expression>");
            }
            CommandResult::Continue
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Type :help for available commands.");
            CommandResult::Continue
        }
    }
}

fn execute_line(line: &str, interpreter: &mut Interpreter) {
    // Попробуем распарсить как одно выражение
    match parse_expr(line) {
        Ok((asg, root_id)) => match interpreter.execute(&asg, root_id) {
            Ok(value) => print_value(&value),
            Err(e) => eprintln!("Runtime error: {}", e),
        },
        Err(_) => {
            // Попробуем как несколько выражений
            match parse(line) {
                Ok((asg, root_ids)) => {
                    if root_ids.is_empty() {
                        return;
                    }
                    let mut last_value = Value::Unit;
                    for root_id in root_ids {
                        match interpreter.execute(&asg, root_id) {
                            Ok(value) => last_value = value,
                            Err(e) => {
                                eprintln!("Runtime error: {}", e);
                                return;
                            }
                        }
                    }
                    print_value(&last_value);
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
    }
}

fn print_value(value: &Value) {
    match value {
        Value::Unit => {} // Не печатаем Unit
        Value::Int(n) => println!("{}", n),
        Value::Float(f) => println!("{}", f),
        Value::Bool(b) => println!("{}", b),
        Value::String(s) => println!("\"{}\"", s),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            println!("[{}]", items.join(", "));
        }
        Value::Record(fields) => {
            let items: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            println!("{{{}}}", items.join(", "));
        }
        Value::Function { params, .. } => {
            println!("<function({})>", params.join(", "));
        }
        Value::Tensor(t) => {
            println!("<tensor {:?}>", t.data.borrow().shape());
        }
        Value::Error(msg) => {
            println!("<error: {}>", msg);
        }
        Value::Dict(dict) => {
            let items: Vec<String> = dict
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            println!("{{{}}}", items.join(", "));
        }
        Value::ComposedFunction(fns) => {
            println!("<composed({} fns)>", fns.len());
        }
        Value::LazySeq(_) => {
            println!("<lazy-seq>");
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Unit => "()".to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_value).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Record(_) => "{...}".to_string(),
        Value::Function { .. } => "<fn>".to_string(),
        Value::Tensor(_) => "<tensor>".to_string(),
        Value::Error(msg) => format!("<error: {}>", msg),
        Value::Dict(dict) => {
            let items: Vec<String> = dict
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        Value::ComposedFunction(fns) => format!("<composed({} fns)>", fns.len()),
        Value::LazySeq(_) => "<lazy-seq>".to_string(),
    }
}

fn show_ast(expr: &str) {
    match parse_expr(expr) {
        Ok((asg, root_id)) => {
            println!("Root ID: {}", root_id);
            println!("Nodes ({}):", asg.nodes.len());
            for node in &asg.nodes {
                println!(
                    "  [{}] {:?} {:?}",
                    node.id,
                    node.node_type,
                    node.get_name().unwrap_or_default()
                );
                for edge in &node.edges {
                    println!("      -> {:?} -> {}", edge.edge_type, edge.target_node_id);
                }
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

fn show_type(expr: &str, _interpreter: &mut Interpreter) {
    match parse_expr(expr) {
        Ok((asg, root_id)) => match asg_lang::type_checker::infer_types(&asg) {
            Ok(type_info) => {
                if let Some(ty) = type_info.get(&root_id) {
                    println!("{:?}", ty);
                } else {
                    println!("Type not found for root node");
                }
            }
            Err(e) => eprintln!("Type error: {}", e),
        },
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

fn load_file(path: &str, interpreter: &mut Interpreter) {
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            return;
        }
    };

    match parse(&source) {
        Ok((asg, root_ids)) => {
            println!("Loading {}...", path);
            let count = root_ids.len();
            for root_id in root_ids {
                if let Err(e) = interpreter.execute(&asg, root_id) {
                    eprintln!("Runtime error: {}", e);
                    return;
                }
            }
            println!("Loaded {} definitions.", count);
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
}

fn show_env(interpreter: &Interpreter) {
    let vars = interpreter.get_variables();
    if vars.is_empty() {
        println!("No variables defined.");
    } else {
        println!("Variables ({}):", vars.len());
        for (name, value) in vars.iter().take(20) {
            println!("  {} = {}", name, format_value(value));
        }
        if vars.len() > 20 {
            println!("  ... and {} more", vars.len() - 20);
        }
    }
}

fn show_functions(interpreter: &Interpreter) {
    let funcs = interpreter.get_functions();
    if funcs.is_empty() {
        println!("No functions defined.");
    } else {
        println!("Functions ({}):", funcs.len());
        for (name, value) in funcs.iter().take(20) {
            match value {
                Value::Function { params, .. } => {
                    println!("  ({} {})", name, params.join(" "));
                }
                _ => println!("  {} = <fn>", name),
            }
        }
        if funcs.len() > 20 {
            println!("  ... and {} more", funcs.len() - 20);
        }
    }
}

/// Выполнить файл.
fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            process::exit(1);
        }
    };

    match parse(&source) {
        Ok((asg, root_ids)) => {
            if root_ids.is_empty() {
                return;
            }

            let mut interpreter = Interpreter::new();
            let mut last_value = Value::Unit;

            // Выполняем все top-level выражения по порядку
            for root_id in root_ids {
                match interpreter.execute(&asg, root_id) {
                    Ok(value) => {
                        last_value = value;
                    }
                    Err(e) => {
                        eprintln!("Runtime error: {}", e);
                        process::exit(1);
                    }
                }
            }

            // Печатаем только последнее значение
            if !matches!(last_value, Value::Unit) {
                print_value(&last_value);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

/// Выполнить выражение.
fn run_expr(expr: &str) {
    match parse_expr(expr) {
        Ok((asg, root_id)) => {
            let mut interpreter = Interpreter::new();
            match interpreter.execute(&asg, root_id) {
                Ok(value) => print_value(&value),
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}
