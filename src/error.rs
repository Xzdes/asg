//! Определения ошибок для ASG.

use crate::nodecodes::EdgeType;
use thiserror::Error;

/// Основной тип `Result` для библиотеки.
pub type ASGResult<T> = Result<T, ASGError>;

/// Перечисление всех возможных ошибок.
#[derive(Error, Debug)]
pub enum ASGError {
    #[error("Node with ID {0} not found in ASG")]
    NodeNotFound(u64),

    #[error("Node {0} is missing required payload")]
    MissingPayload(u64),

    #[error("Node {0} has invalid payload (e.g., wrong size)")]
    InvalidPayload(u64),

    #[error("Node {0} is missing required edge of type {1:?}")]
    MissingEdge(u64, EdgeType),

    #[error("Type mismatch during execution: {0}")]
    TypeError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    // === Новые ошибки для расширенной функциональности ===
    #[error("Effect error: {0}")]
    Effect(String),

    #[error("Compilation error: {0}")]
    CompilationError(String),

    #[error("Type inference error: {0}")]
    TypeInferenceError(String),

    #[error("Unification failed: cannot unify {0} with {1}")]
    UnificationError(String, String),

    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    #[error("Concurrency error: {0}")]
    Concurrency(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Module error: {0}")]
    ModuleError(String),

    #[error("Circular import detected: {0}")]
    CircularImport(String),
}
