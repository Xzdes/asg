//! Загрузчик модулей.
//!
//! Отвечает за чтение, парсинг и загрузку модулей в реестр.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asg::{ASG, NodeID};
use crate::error::{ASGError, ASGResult};
use crate::interpreter::Value;
use crate::nodecodes::{NodeType, EdgeType};
use crate::parser;

use super::{ExportedDef, Module, ModuleRegistry, ModuleResolver};

/// Загрузчик модулей.
#[derive(Debug)]
pub struct ModuleLoader {
    /// Резолвер путей
    resolver: ModuleResolver,
    /// Реестр загруженных модулей
    registry: ModuleRegistry,
    /// Модули в процессе загрузки (для детекции циклов)
    loading: HashSet<String>,
}

impl ModuleLoader {
    /// Создать новый загрузчик.
    pub fn new() -> Self {
        Self {
            resolver: ModuleResolver::new(),
            registry: ModuleRegistry::new(),
            loading: HashSet::new(),
        }
    }

    /// Создать загрузчик с путями поиска.
    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            resolver: ModuleResolver::with_search_paths(paths.clone()),
            registry: ModuleRegistry::with_search_paths(paths),
            loading: HashSet::new(),
        }
    }

    /// Добавить путь поиска модулей.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.resolver.add_search_path(path.clone());
        self.registry.add_search_path(path);
    }

    /// Установить путь к stdlib.
    pub fn set_stdlib_path(&mut self, path: PathBuf) {
        self.resolver.set_stdlib_path(path);
    }

    /// Загрузить модуль по имени.
    pub fn load(&mut self, module_name: &str) -> ASGResult<&Module> {
        // Проверяем, не загружен ли уже
        if self.registry.is_loaded(module_name) {
            return self.registry.get(module_name).ok_or_else(|| {
                ASGError::ModuleNotFound(module_name.to_string())
            });
        }

        // Проверяем циклические импорты
        if self.loading.contains(module_name) {
            return Err(ASGError::CircularImport(module_name.to_string()));
        }

        // Отмечаем как загружаемый
        self.loading.insert(module_name.to_string());

        // Разрешаем путь
        let path = self.resolver.resolve(module_name)?;

        // Загружаем и парсим
        let module = self.load_from_path(module_name, &path)?;

        // Регистрируем модуль
        self.registry.register(module);

        // Убираем из загружаемых
        self.loading.remove(module_name);

        self.registry.get(module_name).ok_or_else(|| {
            ASGError::ModuleNotFound(module_name.to_string())
        })
    }

    /// Загрузить модуль из файла.
    pub fn load_from_path(&mut self, module_name: &str, path: &Path) -> ASGResult<Module> {
        // Читаем файл
        let source = fs::read_to_string(path).map_err(|e| {
            ASGError::IoError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        // Парсим
        let (asg, root_ids) = parser::parse(&source).map_err(|e| {
            ASGError::ParseError(format!("Parse error in {}: {}", path.display(), e))
        })?;

        // Создаём модуль
        let mut module = Module::from_file(module_name.to_string(), path.to_path_buf(), asg.clone());

        // Обрабатываем объявления модуля
        self.process_module_declarations(&mut module, &asg, &root_ids)?;

        Ok(module)
    }

    /// Обработать объявления в модуле.
    fn process_module_declarations(
        &mut self,
        module: &mut Module,
        asg: &ASG,
        root_ids: &[NodeID],
    ) -> ASGResult<()> {
        for &node_id in root_ids {
            let node = asg.find_node(node_id).ok_or_else(|| {
                ASGError::NodeNotFound(node_id)
            })?;

            match node.node_type {
                // Объявление модуля: (module name ...)
                NodeType::Module => {
                    self.process_module_declaration(module, asg, node_id)?;
                }

                // Экспорт: (export name1 name2 ...)
                NodeType::Export => {
                    self.process_export_declaration(module, asg, node_id)?;
                }

                // Импорт: (import "module" ...)
                NodeType::Import => {
                    self.process_import_declaration(module, asg, node_id)?;
                }

                // Определение функции
                NodeType::Function => {
                    self.process_function_definition(module, asg, node_id)?;
                }

                // Определение переменной
                NodeType::Variable => {
                    self.process_variable_definition(module, asg, node_id)?;
                }

                _ => {
                    // Другие узлы пропускаем
                }
            }
        }

        Ok(())
    }

    /// Обработать объявление модуля.
    fn process_module_declaration(
        &mut self,
        module: &mut Module,
        asg: &ASG,
        node_id: NodeID,
    ) -> ASGResult<()> {
        let node = asg.find_node(node_id).ok_or_else(|| {
            ASGError::NodeNotFound(node_id)
        })?;

        // Имя модуля из payload
        if let Some(ref payload) = node.payload {
            let name = String::from_utf8_lossy(payload).to_string();
            module.name = name;
        }

        // Обрабатываем содержимое модуля
        for edge in &node.edges {
            if edge.edge_type == EdgeType::ModuleContent {
                // Рекурсивно обрабатываем содержимое
                self.process_module_declarations(
                    module,
                    asg,
                    &[edge.target_node_id],
                )?;
            }
        }

        Ok(())
    }

    /// Обработать объявление экспорта.
    fn process_export_declaration(
        &mut self,
        module: &mut Module,
        asg: &ASG,
        node_id: NodeID,
    ) -> ASGResult<()> {
        let node = asg.find_node(node_id).ok_or_else(|| {
            ASGError::NodeNotFound(node_id)
        })?;

        let mut exports = Vec::new();

        // Имена экспортов из дочерних узлов или payload
        for edge in &node.edges {
            if edge.edge_type == EdgeType::ApplicationArgument {
                if let Some(name_node) = asg.find_node(edge.target_node_id) {
                    if let Some(ref payload) = name_node.payload {
                        let name = String::from_utf8_lossy(payload).to_string();
                        exports.push(name);
                    }
                }
            }
        }

        // Если экспорты из payload (старый формат)
        if exports.is_empty() {
            if let Some(ref payload) = node.payload {
                let names_str = String::from_utf8_lossy(payload);
                for name in names_str.split_whitespace() {
                    exports.push(name.to_string());
                }
            }
        }

        module.set_explicit_exports(exports);
        Ok(())
    }

    /// Обработать объявление импорта.
    fn process_import_declaration(
        &mut self,
        _module: &mut Module,
        asg: &ASG,
        node_id: NodeID,
    ) -> ASGResult<()> {
        let node = asg.find_node(node_id).ok_or_else(|| {
            ASGError::NodeNotFound(node_id)
        })?;

        // Получаем имя импортируемого модуля
        let import_name = node.payload.as_ref().map(|p| {
            String::from_utf8_lossy(p).to_string()
        }).ok_or_else(|| {
            ASGError::ModuleError("Import missing module name".to_string())
        })?;

        // Загружаем импортируемый модуль (рекурсивно)
        self.load(&import_name)?;

        Ok(())
    }

    /// Обработать определение функции.
    fn process_function_definition(
        &mut self,
        module: &mut Module,
        asg: &ASG,
        node_id: NodeID,
    ) -> ASGResult<()> {
        let node = asg.find_node(node_id).ok_or_else(|| {
            ASGError::NodeNotFound(node_id)
        })?;

        // Получаем имя функции
        let name = node.payload.as_ref().map(|p| {
            String::from_utf8_lossy(p).to_string()
        }).ok_or_else(|| {
            ASGError::ModuleError("Function missing name".to_string())
        })?;

        // Получаем параметры и тело
        let mut params = Vec::new();
        let mut body_id = None;

        for edge in &node.edges {
            match edge.edge_type {
                EdgeType::FunctionParameter => {
                    if let Some(param_node) = asg.find_node(edge.target_node_id) {
                        if let Some(ref payload) = param_node.payload {
                            params.push(String::from_utf8_lossy(payload).to_string());
                        }
                    }
                }
                EdgeType::FunctionBody => {
                    body_id = Some(edge.target_node_id);
                }
                _ => {}
            }
        }

        let body_id = body_id.ok_or_else(|| {
            ASGError::ModuleError(format!("Function {} missing body", name))
        })?;

        // Добавляем определение
        module.definitions.insert(name.clone(), node_id);

        // Добавляем экспорт
        module.add_export(name, ExportedDef::Function {
            params,
            body_id,
            asg: asg.clone(),
        });

        Ok(())
    }

    /// Обработать определение переменной.
    fn process_variable_definition(
        &mut self,
        module: &mut Module,
        asg: &ASG,
        node_id: NodeID,
    ) -> ASGResult<()> {
        let node = asg.find_node(node_id).ok_or_else(|| {
            ASGError::NodeNotFound(node_id)
        })?;

        // Получаем имя переменной
        let name = node.payload.as_ref().map(|p| {
            String::from_utf8_lossy(p).to_string()
        }).ok_or_else(|| {
            ASGError::ModuleError("Variable missing name".to_string())
        })?;

        // Добавляем в определения
        module.definitions.insert(name.clone(), node_id);

        // Пока не вычисляем значение — оставляем как Unit
        module.add_export(name, ExportedDef::Variable(Value::Unit));

        Ok(())
    }

    /// Получить реестр модулей.
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    /// Получить реестр модулей (mutable).
    pub fn registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }

    /// Получить резолвер.
    pub fn resolver(&self) -> &ModuleResolver {
        &self.resolver
    }

    /// Получить резолвер (mutable).
    pub fn resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.resolver
    }

    /// Получить экспорт из модуля.
    pub fn get_export(&mut self, module_name: &str, export_name: &str) -> ASGResult<&ExportedDef> {
        // Загружаем модуль если не загружен
        if !self.registry.is_loaded(module_name) {
            self.load(module_name)?;
        }

        self.registry.get_export(module_name, export_name).ok_or_else(|| {
            ASGError::ModuleError(format!(
                "Export '{}' not found in module '{}'",
                export_name, module_name
            ))
        })
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.registry.count(), 0);
    }

    #[test]
    fn test_load_simple_module() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("simple.asg");

        let mut file = fs::File::create(&module_path).unwrap();
        writeln!(file, "(fn hello () \"Hello, World!\")").unwrap();

        let mut loader = ModuleLoader::with_search_paths(vec![dir.path().to_path_buf()]);
        let result = loader.load("simple");

        assert!(result.is_ok());
        assert!(loader.registry.is_loaded("simple"));
    }

    #[test]
    fn test_circular_import_detection() {
        let mut loader = ModuleLoader::new();
        loader.loading.insert("circular".to_string());

        let result = loader.load("circular");
        assert!(matches!(result, Err(ASGError::CircularImport(_))));
    }
}
