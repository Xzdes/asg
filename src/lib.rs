//! # ASG Core
//!
//! Основная библиотека для языка программирования ASG.
//!
//! ## Основные модули
//!
//! - [`asg`] - Абстрактный Синтаксический Граф (ASG)
//! - [`nodecodes`] - Типы узлов и рёбер
//! - [`parser`] - S-Expression парсер
//! - [`interpreter`] - Интерпретатор ASG
//! - [`type_checker`] - Проверка и вывод типов
//! - [`types`] - Система типов ASG
//! - [`llvm_backend`] - Компиляция в LLVM IR (требует feature `llvm_backend`)
//!
//! ## Features
//!
//! - `llvm_backend` - Включает компиляцию в нативный код через LLVM
//! - `proofs` - Включает систему доказательств через Z3
//!
//! ## Пример использования парсера
//!
//! ```rust,ignore
//! use asg_lang::parser::{parse, parse_expr};
//! use asg_lang::Interpreter;
//!
//! // Парсинг выражения
//! let (asg, root_id) = parse_expr("(+ 1 2)").unwrap();
//!
//! // Выполнение
//! let mut interpreter = Interpreter::new();
//! let result = interpreter.execute(&asg, root_id).unwrap();
//! ```

// === Основные модули ===
pub mod asg;
pub mod error;
pub mod interpreter;
pub mod nodecodes;
pub mod ops;
pub mod parser;
pub mod runtime;
pub mod type_checker;
pub mod types;

// === Компиляторные бэкенды ===
pub mod c_backend;
pub mod compiler;
pub mod js_backend;
pub mod llvm_backend;
pub mod wasm; // WASM GC и runtime
pub mod wasm_backend;

// === GUI модуль (requires feature 'gui') ===
#[cfg(feature = "gui")]
pub mod gui;

// === Дополнительные модули ===
pub mod ai_api;
pub mod concurrency;
pub mod concurrency_async;
pub mod effects;
pub mod ffi;
pub mod macros;
pub mod modules;

// === Система доказательств ===
pub mod proof;
pub mod proof_dsl;
pub mod proof_smt;

// === Re-exports для удобства ===
pub use asg::{Edge, Node, NodeID, ASG};
pub use error::{ASGError, ASGResult};
pub use interpreter::{Interpreter, Value};
pub use nodecodes::{EdgeType, NodeType};
pub use parser::{parse, parse_expr};
pub use type_checker::{check_types, infer_types, TypeChecker};
pub use types::SynType;
