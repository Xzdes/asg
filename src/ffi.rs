//! Модуль `ffi`
//!
//! Поддержка FFI (Foreign Function Interface) в ASG:
//! - ForeignFunctionDecl
//! - ForeignBlock
//!
//! Заглушки, без использования unsafe.

use serde::{Deserialize, Serialize};

use crate::ASGResult;

/// Декларация внешней функции.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignFunctionDecl {
    /// Имя функции.
    pub name: String,
    /// ABI (например, "C").
    pub abi: String,
    /// Сигнатура функции.
    pub signature: String,
}

/// Блок внешнего кода (заглушка).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignBlock {
    /// Содержимое блока (например, C-код).
    pub code: String,
}

/// Проверка безопасности FFI.
/// (На данном этапе просто заглушка.)
pub fn check_ffi_safety(_decl: &ForeignFunctionDecl) -> ASGResult<()> {
    println!("Checking FFI safety for function `{}`...", _decl.name);
    Ok(())
}
