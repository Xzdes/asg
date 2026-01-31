//! Модуль парсера S-Expression для ASG.
//!
//! Этот модуль предоставляет парсер для S-Expression синтаксиса,
//! оптимизированного для обработки LLM.
//!
//! # Синтаксис
//!
//! ```lisp
//! ; Литералы
//! 42              ; Int
//! 3.14            ; Float
//! true false      ; Bool
//! "hello"         ; String
//! ()              ; Unit
//!
//! ; Арифметика
//! (+ a b)         ; сложение
//! (- a b)         ; вычитание
//! (* a b)         ; умножение
//! (/ a b)         ; деление
//!
//! ; Переменные
//! (let x 42)      ; объявление
//! x               ; ссылка
//! (set x 100)     ; присваивание
//!
//! ; Условия
//! (if cond then else)
//!
//! ; Функции
//! (fn name (params) body)
//! (lambda (x) (* x x))
//! (func arg1 arg2)  ; вызов
//! ```
//!
//! # Пример
//!
//! ```rust,ignore
//! use asg_lang::parser::parse;
//!
//! let source = "(+ 1 2)";
//! let asg = parse(source).unwrap();
//! ```

pub mod builder;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

pub use builder::AsgBuilder;
pub use error::ParseError;
pub use lexer::Lexer;
pub use parser::{Atom, Parser, SExpr};
pub use token::{Span, Spanned, Token};

use crate::asg::{NodeID, ASG};
use crate::error::ASGResult;

/// Парсит исходный код в ASG.
///
/// # Аргументы
///
/// * `source` — исходный код на языке ASG (S-Expression).
///
/// # Возвращает
///
/// Кортеж (ASG, Vec<NodeID>) — ASG и список ID корневых узлов (top-level выражений).
///
/// # Пример
///
/// ```rust,ignore
/// use asg_lang::parser::parse;
///
/// let (asg, root_ids) = parse("(let x 1) x").unwrap();
/// ```
pub fn parse(source: &str) -> ASGResult<(ASG, Vec<NodeID>)> {
    let mut parser = Parser::new(source);
    let exprs = parser
        .parse_all()
        .map_err(|e| crate::error::ASGError::ParseError(e.to_string()))?;

    let builder = AsgBuilder::new();
    builder
        .build(exprs)
        .map_err(|e| crate::error::ASGError::ParseError(e.to_string()))
}

/// Парсит одно выражение и возвращает ASG с ID корневого узла.
///
/// # Аргументы
///
/// * `source` — исходный код одного выражения.
///
/// # Возвращает
///
/// Кортеж (ASG, root_id) или ошибку.
pub fn parse_expr(source: &str) -> ASGResult<(ASG, NodeID)> {
    let mut parser = Parser::new(source);
    let expr = parser
        .parse_sexpr()
        .map_err(|e| crate::error::ASGError::ParseError(e.to_string()))?;

    let builder = AsgBuilder::new();
    builder
        .build_single(&expr)
        .map_err(|e| crate::error::ASGError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::Interpreter;

    #[test]
    fn test_parse_and_execute_add() {
        let (asg, root_id) = parse_expr("(+ 5 8)").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::Int(n) => assert_eq!(n, 13),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_parse_and_execute_nested() {
        let (asg, root_id) = parse_expr("(* (+ 2 3) 4)").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::Int(n) => assert_eq!(n, 20),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_parse_and_execute_comparison() {
        let (asg, root_id) = parse_expr("(< 5 10)").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_parse_string() {
        let (asg, root_id) = parse_expr(r#""hello world""#).unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::String(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_parse_bool() {
        let (asg, root_id) = parse_expr("true").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_parse_unit() {
        let (asg, root_id) = parse_expr("()").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        assert!(matches!(result, crate::interpreter::Value::Unit));
    }

    #[test]
    fn test_parse_if() {
        let (asg, root_id) = parse_expr("(if true 42 0)").unwrap();

        let mut interpreter = Interpreter::new();
        let result = interpreter.execute(&asg, root_id).unwrap();

        match result {
            crate::interpreter::Value::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_parse_let() {
        let (asg, root_ids) = parse("(let x 10) x").unwrap();

        let mut interpreter = Interpreter::new();

        // Выполняем все выражения по порядку
        let mut result = crate::interpreter::Value::Unit;
        for root_id in root_ids {
            result = interpreter.execute(&asg, root_id).unwrap();
        }

        match result {
            crate::interpreter::Value::Int(n) => assert_eq!(n, 10),
            _ => panic!("Expected Int"),
        }
    }
}
