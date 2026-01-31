//! Система модулей ASG.
//!
//! Модули позволяют организовывать код и управлять видимостью.
//!
//! ## Синтаксис
//!
//! ```lisp
//! ; Объявление модуля
//! (module math
//!   (export square cube PI)
//!   (let PI 3.14159)
//!   (fn square (x) (* x x))
//!   (fn cube (x) (* x x x)))
//!
//! ; Импорт
//! (import "math")           ; всё
//! (import "math" :as m)     ; с алиасом
//! (import "math" :only (square PI))  ; выборочно
//! ```

mod registry;
mod resolver;
mod loader;

pub use registry::{Module, ModuleRegistry};
pub use resolver::{ModuleResolver, ResolveStrategy};
pub use loader::ModuleLoader;

use std::path::PathBuf;

use crate::asg::{ASG, NodeID};
use crate::interpreter::Value;

/// Экспортируемое определение из модуля.
#[derive(Debug, Clone)]
pub enum ExportedDef {
    /// Функция (имя, параметры, body_id, asg)
    Function {
        params: Vec<String>,
        body_id: NodeID,
        asg: ASG,
    },
    /// Переменная/константа
    Variable(Value),
}

/// Конфигурация модульной системы.
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    /// Пути поиска модулей
    pub search_paths: Vec<PathBuf>,
    /// Путь к стандартной библиотеке
    pub stdlib_path: Option<PathBuf>,
    /// Кэшировать загруженные модули
    pub cache_modules: bool,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            stdlib_path: None,
            cache_modules: true,
        }
    }
}

/// Проверить ASG модуля (заглушка для совместимости).
pub fn check_module(asg: &ASG) {
    println!("Modules: checking module with {} nodes.", asg.nodes.len());
}
