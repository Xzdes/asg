//! Демонстрация S-Expression парсера ASG.
//!
//! Запуск: cargo run --example parser_demo

use asg::interpreter::Interpreter;
use asg::parser::{parse, parse_expr};

fn main() {
    println!("=== ASG S-Expression Parser Demo ===\n");

    // Пример 1: Простое выражение
    println!("1. Простое выражение: (+ 5 8)");
    let (asg, root_id) = parse_expr("(+ 5 8)").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 2: Вложенные операции
    println!("2. Вложенные операции: (* (+ 2 3) (- 10 4))");
    let (asg, root_id) = parse_expr("(* (+ 2 3) (- 10 4))").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 3: Сравнение
    println!("3. Сравнение: (<= 5 10)");
    let (asg, root_id) = parse_expr("(<= 5 10)").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 4: Логические операции
    println!("4. Логика: (and true (not false))");
    let (asg, root_id) = parse_expr("(and true (not false))").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 5: Условное выражение
    println!("5. Условие: (if (> 10 5) 100 0)");
    let (asg, root_id) = parse_expr("(if (> 10 5) 100 0)").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 6: Переменные
    println!("6. Переменные: (let x 42) x");
    let (asg, root_ids) = parse("(let x 42) x").unwrap();
    let mut interpreter = Interpreter::new();
    let mut result = asg::interpreter::Value::Unit;
    for root_id in root_ids {
        result = interpreter.execute(&asg, root_id).unwrap();
    }
    println!("   Результат: {:?}\n", result);

    // Пример 7: Массив
    println!("7. Массив: (array 1 2 3 4 5)");
    let (asg, root_id) = parse_expr("(array 1 2 3 4 5)").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 8: Unit
    println!("8. Unit: ()");
    let (asg, root_id) = parse_expr("()").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 9: Строка
    println!(r#"9. Строка: "Hello, ASG!""#);
    let (asg, root_id) = parse_expr(r#""Hello, ASG!""#).unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    // Пример 10: Float
    println!("10. Float: (+ 3.14 2.86)");
    let (asg, root_id) = parse_expr("(+ 3.14 2.86)").unwrap();
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&asg, root_id).unwrap();
    println!("   Результат: {:?}\n", result);

    println!("=== Демонстрация завершена ===");
}
