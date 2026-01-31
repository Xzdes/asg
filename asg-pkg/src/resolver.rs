//! Dependency resolver.

use crate::manifest::{Dependency, Manifest};
use crate::registry::RegistryClient;
use semver::{Version, VersionReq};
use std::collections::{HashMap, HashSet};

/// Разрешённая зависимость.
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    /// Имя пакета
    pub name: String,
    /// Разрешённая версия
    pub version: String,
    /// Контрольная сумма
    pub checksum: Option<String>,
    /// Транзитивные зависимости
    pub dependencies: Vec<String>,
}

/// Граф зависимостей.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Все разрешённые зависимости
    pub resolved: HashMap<String, ResolvedDependency>,
    /// Порядок установки (топологическая сортировка)
    pub install_order: Vec<String>,
}

/// Резолвер зависимостей.
pub struct Resolver {
    /// Клиент реестра
    registry: RegistryClient,
    /// Кэш версий
    version_cache: HashMap<String, Vec<String>>,
}

impl Resolver {
    /// Создать новый резолвер.
    pub fn new(registry: RegistryClient) -> Self {
        Self {
            registry,
            version_cache: HashMap::new(),
        }
    }

    /// Разрешить все зависимости манифеста.
    pub fn resolve(&mut self, manifest: &Manifest) -> Result<DependencyGraph, ResolverError> {
        let mut graph = DependencyGraph::default();
        let mut visited = HashSet::new();

        // Разрешаем основные зависимости
        for (name, dep) in &manifest.dependencies {
            self.resolve_dependency(name, dep, &mut graph, &mut visited)?;
        }

        // Вычисляем порядок установки
        graph.install_order = self.topological_sort(&graph)?;

        Ok(graph)
    }

    /// Разрешить одну зависимость.
    fn resolve_dependency(
        &mut self,
        name: &str,
        dep: &Dependency,
        graph: &mut DependencyGraph,
        visited: &mut HashSet<String>,
    ) -> Result<(), ResolverError> {
        // Проверяем циклические зависимости
        if visited.contains(name) {
            return Err(ResolverError::CircularDependency(name.to_string()));
        }

        // Уже разрешено?
        if graph.resolved.contains_key(name) {
            return Ok(());
        }

        visited.insert(name.to_string());

        // Парсим версию
        let version_req = self.parse_version_req(dep.version())?;

        // Получаем доступные версии
        let available_versions = self.get_available_versions(name)?;

        // Находим подходящую версию
        let resolved_version = self
            .find_matching_version(&version_req, &available_versions)
            .ok_or_else(|| {
                ResolverError::NoMatchingVersion(name.to_string(), dep.version().to_string())
            })?;

        // Получаем информацию о версии
        let version_info = self
            .registry
            .get_version(name, &resolved_version)
            .map_err(|e| ResolverError::Registry(e.to_string()))?;

        // Сохраняем зависимость
        let mut dep_names = Vec::new();

        // Рекурсивно разрешаем транзитивные зависимости
        for (dep_name, dep_version) in &version_info.dependencies {
            dep_names.push(dep_name.clone());
            let transitive_dep = Dependency::Simple(dep_version.clone());
            self.resolve_dependency(dep_name, &transitive_dep, graph, visited)?;
        }

        graph.resolved.insert(
            name.to_string(),
            ResolvedDependency {
                name: name.to_string(),
                version: resolved_version,
                checksum: Some(version_info.checksum),
                dependencies: dep_names,
            },
        );

        visited.remove(name);
        Ok(())
    }

    /// Парсинг версии.
    fn parse_version_req(&self, version_str: &str) -> Result<VersionReq, ResolverError> {
        // Поддерживаем разные форматы:
        // "1.0.0" -> "=1.0.0" (точная версия)
        // "^1.0.0" -> ">=1.0.0, <2.0.0" (совместимая)
        // "~1.0.0" -> ">=1.0.0, <1.1.0" (патч-совместимая)
        // ">=1.0.0" -> как есть
        // "*" -> любая версия

        let normalized = if version_str.starts_with(['=', '^', '~', '>', '<', '*']) {
            version_str.to_string()
        } else {
            format!("^{}", version_str) // По умолчанию используем ^
        };

        VersionReq::parse(&normalized)
            .map_err(|e| ResolverError::InvalidVersion(version_str.to_string(), e.to_string()))
    }

    /// Получить доступные версии пакета.
    fn get_available_versions(&mut self, name: &str) -> Result<Vec<String>, ResolverError> {
        // Проверяем кэш
        if let Some(versions) = self.version_cache.get(name) {
            return Ok(versions.clone());
        }

        // Запрашиваем из реестра
        let package_info = self
            .registry
            .get_package(name)
            .map_err(|e| ResolverError::Registry(e.to_string()))?;

        let versions: Vec<String> = package_info
            .versions
            .iter()
            .filter(|v| !v.yanked)
            .map(|v| v.version.clone())
            .collect();

        self.version_cache.insert(name.to_string(), versions.clone());

        Ok(versions)
    }

    /// Найти версию, соответствующую требованию.
    fn find_matching_version(
        &self,
        req: &VersionReq,
        available: &[String],
    ) -> Option<String> {
        // Парсим и сортируем версии
        let mut parsed: Vec<(Version, &str)> = available
            .iter()
            .filter_map(|v| Version::parse(v).ok().map(|parsed| (parsed, v.as_str())))
            .collect();

        // Сортируем по убыванию (новые первые)
        parsed.sort_by(|a, b| b.0.cmp(&a.0));

        // Находим первую подходящую
        for (version, original) in parsed {
            if req.matches(&version) {
                return Some(original.to_string());
            }
        }

        None
    }

    /// Топологическая сортировка для определения порядка установки.
    fn topological_sort(&self, graph: &DependencyGraph) -> Result<Vec<String>, ResolverError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();

        fn visit(
            name: &str,
            graph: &DependencyGraph,
            visited: &mut HashSet<String>,
            in_progress: &mut HashSet<String>,
            result: &mut Vec<String>,
        ) -> Result<(), ResolverError> {
            if visited.contains(name) {
                return Ok(());
            }

            if in_progress.contains(name) {
                return Err(ResolverError::CircularDependency(name.to_string()));
            }

            in_progress.insert(name.to_string());

            if let Some(dep) = graph.resolved.get(name) {
                for child in &dep.dependencies {
                    visit(child, graph, visited, in_progress, result)?;
                }
            }

            in_progress.remove(name);
            visited.insert(name.to_string());
            result.push(name.to_string());

            Ok(())
        }

        for name in graph.resolved.keys() {
            visit(name, graph, &mut visited, &mut in_progress, &mut result)?;
        }

        Ok(result)
    }
}

/// Ошибки резолвера.
#[derive(Debug)]
pub enum ResolverError {
    Registry(String),
    InvalidVersion(String, String),
    NoMatchingVersion(String, String),
    CircularDependency(String),
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverError::Registry(e) => write!(f, "Registry error: {}", e),
            ResolverError::InvalidVersion(v, e) => {
                write!(f, "Invalid version '{}': {}", v, e)
            }
            ResolverError::NoMatchingVersion(name, req) => {
                write!(f, "No version of {} matches requirement {}", name, req)
            }
            ResolverError::CircularDependency(name) => {
                write!(f, "Circular dependency detected: {}", name)
            }
        }
    }
}

impl std::error::Error for ResolverError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_req() {
        let resolver = Resolver::new(RegistryClient::new(None));

        // Точная версия
        let req = resolver.parse_version_req("1.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.0.5").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_find_matching_version() {
        let resolver = Resolver::new(RegistryClient::new(None));
        let req = VersionReq::parse("^1.0.0").unwrap();
        let available = vec![
            "0.9.0".to_string(),
            "1.0.0".to_string(),
            "1.5.0".to_string(),
            "2.0.0".to_string(),
        ];

        let result = resolver.find_matching_version(&req, &available);
        assert_eq!(result, Some("1.5.0".to_string())); // Наибольшая совместимая
    }
}
