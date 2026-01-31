//! Разрешение путей модулей.
//!
//! Отвечает за поиск файлов модулей по имени и разрешение относительных путей.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{ASGError, ASGResult};

/// Стратегия разрешения имён модулей.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveStrategy {
    /// Сначала локальные, потом stdlib
    LocalFirst,
    /// Сначала stdlib, потом локальные
    StdlibFirst,
    /// Только локальные
    LocalOnly,
    /// Только stdlib
    StdlibOnly,
}

impl Default for ResolveStrategy {
    fn default() -> Self {
        Self::LocalFirst
    }
}

/// Резолвер модулей.
#[derive(Debug)]
pub struct ModuleResolver {
    /// Пути поиска локальных модулей
    search_paths: Vec<PathBuf>,
    /// Путь к stdlib
    stdlib_path: Option<PathBuf>,
    /// Стратегия разрешения
    strategy: ResolveStrategy,
    /// Кэш разрешённых путей
    cache: HashMap<String, PathBuf>,
    /// Расширения файлов модулей
    extensions: Vec<String>,
}

impl ModuleResolver {
    /// Создать новый резолвер.
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            stdlib_path: None,
            strategy: ResolveStrategy::default(),
            cache: HashMap::new(),
            extensions: vec!["syn".to_string(), "asg".to_string()],
        }
    }

    /// Создать резолвер с путями поиска.
    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: paths,
            stdlib_path: None,
            strategy: ResolveStrategy::default(),
            cache: HashMap::new(),
            extensions: vec!["syn".to_string(), "asg".to_string()],
        }
    }

    /// Установить путь к stdlib.
    pub fn set_stdlib_path(&mut self, path: PathBuf) {
        self.stdlib_path = Some(path);
    }

    /// Добавить путь поиска.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Установить стратегию разрешения.
    pub fn set_strategy(&mut self, strategy: ResolveStrategy) {
        self.strategy = strategy;
    }

    /// Очистить кэш.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Разрешить имя модуля в путь к файлу.
    pub fn resolve(&mut self, module_name: &str) -> ASGResult<PathBuf> {
        // Проверяем кэш
        if let Some(path) = self.cache.get(module_name) {
            return Ok(path.clone());
        }

        // Разрешаем по стратегии
        let result = match self.strategy {
            ResolveStrategy::LocalFirst => {
                self.resolve_local(module_name)
                    .or_else(|_| self.resolve_stdlib(module_name))
            }
            ResolveStrategy::StdlibFirst => {
                self.resolve_stdlib(module_name)
                    .or_else(|_| self.resolve_local(module_name))
            }
            ResolveStrategy::LocalOnly => self.resolve_local(module_name),
            ResolveStrategy::StdlibOnly => self.resolve_stdlib(module_name),
        };

        // Кэшируем успешный результат
        if let Ok(ref path) = result {
            self.cache.insert(module_name.to_string(), path.clone());
        }

        result
    }

    /// Разрешить локальный модуль.
    fn resolve_local(&self, module_name: &str) -> ASGResult<PathBuf> {
        for search_path in &self.search_paths {
            if let Some(path) = self.find_module_in_dir(search_path, module_name) {
                return Ok(path);
            }
        }

        Err(ASGError::ModuleNotFound(module_name.to_string()))
    }

    /// Разрешить stdlib модуль.
    fn resolve_stdlib(&self, module_name: &str) -> ASGResult<PathBuf> {
        let stdlib_path = self.stdlib_path.as_ref().ok_or_else(|| {
            ASGError::ModuleNotFound(format!("stdlib not configured for: {}", module_name))
        })?;

        // Для stdlib путей типа "std/math" или "math"
        let normalized = if module_name.starts_with("std/") {
            &module_name[4..]
        } else {
            module_name
        };

        self.find_module_in_dir(stdlib_path, normalized)
            .ok_or_else(|| ASGError::ModuleNotFound(module_name.to_string()))
    }

    /// Найти модуль в директории.
    fn find_module_in_dir(&self, dir: &Path, module_name: &str) -> Option<PathBuf> {
        // Преобразуем путь модуля: "std/math" -> "std/math"
        // "math.utils" -> "math/utils"
        let module_path = module_name.replace('.', "/");

        // Варианты путей к файлу модуля
        let candidates: Vec<PathBuf> = self
            .extensions
            .iter()
            .flat_map(|ext| {
                vec![
                    // module_name.asg
                    dir.join(format!("{}.{}", module_path, ext)),
                    // module_name/mod.asg
                    dir.join(&module_path).join(format!("mod.{}", ext)),
                    // module_name/index.asg
                    dir.join(&module_path).join(format!("index.{}", ext)),
                ]
            })
            .collect();

        for candidate in candidates {
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }

        None
    }

    /// Разрешить относительный импорт.
    ///
    /// Относительные импорты начинаются с `.` или `..`:
    /// - `./sibling` - модуль в той же директории
    /// - `../parent` - модуль в родительской директории
    pub fn resolve_relative(&mut self, from_file: &Path, module_name: &str) -> ASGResult<PathBuf> {
        let base_dir = from_file.parent().ok_or_else(|| {
            ASGError::ModuleNotFound(format!(
                "cannot resolve relative import from: {:?}",
                from_file
            ))
        })?;

        // Нормализуем путь
        let resolved = if module_name.starts_with("./") {
            base_dir.join(&module_name[2..])
        } else if module_name.starts_with("../") {
            let mut current = base_dir.to_path_buf();
            let mut remaining = module_name;

            while remaining.starts_with("../") {
                current = current.parent().ok_or_else(|| {
                    ASGError::ModuleNotFound(format!(
                        "cannot go above root directory: {}",
                        module_name
                    ))
                })?.to_path_buf();
                remaining = &remaining[3..];
            }

            current.join(remaining)
        } else {
            // Не относительный путь — используем обычное разрешение
            return self.resolve(module_name);
        };

        // Ищем файл с расширением
        self.find_module_in_dir(resolved.parent().unwrap_or(Path::new(".")),
                                resolved.file_name().unwrap_or_default().to_str().unwrap_or(""))
            .ok_or_else(|| ASGError::ModuleNotFound(module_name.to_string()))
    }

    /// Проверить, является ли путь stdlib модулем.
    pub fn is_stdlib_module(&self, module_name: &str) -> bool {
        module_name.starts_with("std/") || module_name.starts_with("std.")
    }

    /// Получить все пути поиска.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Получить путь к stdlib.
    pub fn stdlib_path(&self) -> Option<&PathBuf> {
        self.stdlib_path.as_ref()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_resolver_creation() {
        let resolver = ModuleResolver::new();
        assert_eq!(resolver.search_paths.len(), 1);
        assert_eq!(resolver.strategy, ResolveStrategy::LocalFirst);
    }

    #[test]
    fn test_resolve_local_module() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("math.asg");
        File::create(&module_path).unwrap().write_all(b"(module math)").unwrap();

        let mut resolver = ModuleResolver::with_search_paths(vec![dir.path().to_path_buf()]);
        let result = resolver.resolve("math");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), module_path);
    }

    #[test]
    fn test_resolve_nested_module() {
        let dir = tempdir().unwrap();
        let nested_dir = dir.path().join("utils");
        fs::create_dir(&nested_dir).unwrap();
        let module_path = nested_dir.join("mod.asg");
        File::create(&module_path).unwrap().write_all(b"(module utils)").unwrap();

        let mut resolver = ModuleResolver::with_search_paths(vec![dir.path().to_path_buf()]);
        let result = resolver.resolve("utils");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), module_path);
    }

    #[test]
    fn test_resolve_not_found() {
        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve("nonexistent_module_xyz");

        assert!(result.is_err());
    }

    #[test]
    fn test_cache() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("cached.asg");
        File::create(&module_path).unwrap().write_all(b"(module cached)").unwrap();

        let mut resolver = ModuleResolver::with_search_paths(vec![dir.path().to_path_buf()]);

        // Первый вызов
        let _ = resolver.resolve("cached");
        assert!(resolver.cache.contains_key("cached"));

        // Очистка кэша
        resolver.clear_cache();
        assert!(!resolver.cache.contains_key("cached"));
    }

    #[test]
    fn test_is_stdlib_module() {
        let resolver = ModuleResolver::new();

        assert!(resolver.is_stdlib_module("std/math"));
        assert!(resolver.is_stdlib_module("std.math"));
        assert!(!resolver.is_stdlib_module("math"));
        assert!(!resolver.is_stdlib_module("local/module"));
    }
}
