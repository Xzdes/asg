//! Реестр модулей.

use std::collections::HashMap;
use std::path::PathBuf;

use super::ExportedDef;
use crate::asg::{NodeID, ASG};

/// Загруженный модуль.
#[derive(Debug, Clone)]
pub struct Module {
    /// Имя модуля
    pub name: String,
    /// Путь к файлу (если загружен из файла)
    pub path: Option<PathBuf>,
    /// Экспортируемые определения
    pub exports: HashMap<String, ExportedDef>,
    /// ASG модуля
    pub asg: ASG,
    /// Все определения (включая приватные)
    pub definitions: HashMap<String, NodeID>,
    /// Явно экспортируемые имена (если None — всё публично)
    pub explicit_exports: Option<Vec<String>>,
}

impl Module {
    /// Создать новый пустой модуль.
    pub fn new(name: String) -> Self {
        Self {
            name,
            path: None,
            exports: HashMap::new(),
            asg: ASG::new(),
            definitions: HashMap::new(),
            explicit_exports: None,
        }
    }

    /// Создать модуль из файла.
    pub fn from_file(name: String, path: PathBuf, asg: ASG) -> Self {
        Self {
            name,
            path: Some(path),
            exports: HashMap::new(),
            asg,
            definitions: HashMap::new(),
            explicit_exports: None,
        }
    }

    /// Добавить экспорт.
    pub fn add_export(&mut self, name: String, def: ExportedDef) {
        self.exports.insert(name, def);
    }

    /// Проверить, экспортируется ли имя.
    pub fn is_exported(&self, name: &str) -> bool {
        match &self.explicit_exports {
            Some(exports) => exports.contains(&name.to_string()),
            None => true, // Если нет явных экспортов — всё публично
        }
    }

    /// Установить список явных экспортов.
    pub fn set_explicit_exports(&mut self, names: Vec<String>) {
        self.explicit_exports = Some(names);
    }

    /// Получить экспорт по имени.
    pub fn get_export(&self, name: &str) -> Option<&ExportedDef> {
        if self.is_exported(name) {
            self.exports.get(name)
        } else {
            None
        }
    }
}

/// Реестр загруженных модулей.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    /// Загруженные модули по имени
    modules: HashMap<String, Module>,
    /// Пути поиска модулей
    search_paths: Vec<PathBuf>,
    /// Кэш путей к файлам модулей
    path_cache: HashMap<String, PathBuf>,
}

impl ModuleRegistry {
    /// Создать новый реестр.
    pub fn new() -> Self {
        Self::default()
    }

    /// Создать реестр с путями поиска.
    pub fn with_search_paths(search_paths: Vec<PathBuf>) -> Self {
        Self {
            modules: HashMap::new(),
            search_paths,
            path_cache: HashMap::new(),
        }
    }

    /// Добавить путь поиска.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Зарегистрировать модуль.
    pub fn register(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Получить модуль по имени.
    pub fn get(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Получить модуль по имени (mutable).
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Module> {
        self.modules.get_mut(name)
    }

    /// Проверить, загружен ли модуль.
    pub fn is_loaded(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Получить все загруженные модули.
    pub fn all_modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
    }

    /// Получить количество загруженных модулей.
    pub fn count(&self) -> usize {
        self.modules.len()
    }

    /// Найти путь к файлу модуля.
    pub fn find_module_path(&self, module_name: &str) -> Option<PathBuf> {
        // Проверяем кэш
        if let Some(path) = self.path_cache.get(module_name) {
            return Some(path.clone());
        }

        // Ищем в путях поиска
        for search_path in &self.search_paths {
            // Пробуем разные варианты имени файла
            let candidates = [
                search_path.join(format!("{}.asg", module_name)),
                search_path.join(module_name).join("mod.asg"),
                search_path.join(format!("{}/index.asg", module_name)),
            ];

            for candidate in &candidates {
                if candidate.exists() {
                    return Some(candidate.clone());
                }
            }
        }

        None
    }

    /// Получить экспорт из модуля.
    pub fn get_export(&self, module_name: &str, export_name: &str) -> Option<&ExportedDef> {
        self.modules.get(module_name)?.get_export(export_name)
    }

    /// Получить все экспорты модуля.
    pub fn get_all_exports(&self, module_name: &str) -> Option<&HashMap<String, ExportedDef>> {
        self.modules.get(module_name).map(|m| &m.exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = Module::new("test".to_string());
        assert_eq!(module.name, "test");
        assert!(module.exports.is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ModuleRegistry::new();
        let module = Module::new("math".to_string());
        registry.register(module);

        assert!(registry.is_loaded("math"));
        assert!(!registry.is_loaded("string"));
    }

    #[test]
    fn test_explicit_exports() {
        let mut module = Module::new("test".to_string());
        module.set_explicit_exports(vec!["public_fn".to_string()]);

        assert!(module.is_exported("public_fn"));
        assert!(!module.is_exported("private_fn"));
    }
}
