//! Package manifest (asg.toml) handling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Файл манифеста проекта.
pub const MANIFEST_FILE: &str = "asg.toml";

/// Манифест пакета (asg.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Информация о пакете
    pub package: Package,

    /// Зависимости
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Dev-зависимости
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,

    /// Настройки сборки
    #[serde(default)]
    pub build: BuildConfig,

    /// Настройки для разных целей
    #[serde(default)]
    pub targets: HashMap<String, TargetConfig>,

    /// Метаданные пакета
    #[serde(default)]
    pub metadata: HashMap<String, toml::Value>,
}

/// Информация о пакете.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Имя пакета
    pub name: String,

    /// Версия
    pub version: String,

    /// Описание
    #[serde(default)]
    pub description: Option<String>,

    /// Авторы
    #[serde(default)]
    pub authors: Vec<String>,

    /// Лицензия
    #[serde(default)]
    pub license: Option<String>,

    /// URL репозитория
    #[serde(default)]
    pub repository: Option<String>,

    /// Домашняя страница
    #[serde(default)]
    pub homepage: Option<String>,

    /// Ключевые слова
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Категории
    #[serde(default)]
    pub categories: Vec<String>,

    /// Точка входа
    #[serde(default)]
    pub entry: Option<String>,

    /// Тип пакета (bin/lib)
    #[serde(default)]
    pub package_type: PackageType,

    /// Минимальная версия ASG
    #[serde(default)]
    pub asg_version: Option<String>,
}

/// Тип пакета.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    #[default]
    Bin,
    Lib,
}

/// Зависимость.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Простая версия
    Simple(String),

    /// Расширенная конфигурация
    Detailed(DetailedDependency),
}

impl Dependency {
    /// Получить версию зависимости.
    pub fn version(&self) -> &str {
        match self {
            Dependency::Simple(v) => v,
            Dependency::Detailed(d) => &d.version,
        }
    }
}

/// Расширенная конфигурация зависимости.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedDependency {
    /// Версия
    pub version: String,

    /// Git репозиторий
    #[serde(default)]
    pub git: Option<String>,

    /// Git ветка
    #[serde(default)]
    pub branch: Option<String>,

    /// Git тег
    #[serde(default)]
    pub tag: Option<String>,

    /// Git коммит
    #[serde(default)]
    pub rev: Option<String>,

    /// Локальный путь
    #[serde(default)]
    pub path: Option<String>,

    /// Фичи для включения
    #[serde(default)]
    pub features: Vec<String>,

    /// Опциональная зависимость
    #[serde(default)]
    pub optional: bool,
}

/// Настройки сборки.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Целевой бэкенд
    #[serde(default)]
    pub target: Option<String>,

    /// Оптимизации
    #[serde(default)]
    pub optimize: bool,

    /// Дебаг-информация
    #[serde(default)]
    pub debug: bool,

    /// Дополнительные флаги
    #[serde(default)]
    pub flags: Vec<String>,
}

/// Настройки для конкретной цели.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetConfig {
    /// Дополнительные зависимости
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    /// Настройки сборки
    #[serde(default)]
    pub build: BuildConfig,
}

impl Manifest {
    /// Создать новый манифест.
    pub fn new(name: &str, is_lib: bool) -> Self {
        Self {
            package: Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: None,
                authors: vec![],
                license: Some("MIT".to_string()),
                repository: None,
                homepage: None,
                keywords: vec![],
                categories: vec![],
                entry: Some(if is_lib {
                    "src/lib.syn".to_string()
                } else {
                    "src/main.syn".to_string()
                }),
                package_type: if is_lib {
                    PackageType::Lib
                } else {
                    PackageType::Bin
                },
                asg_version: Some(">=0.7.0".to_string()),
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            build: BuildConfig::default(),
            targets: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Загрузить манифест из файла.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| ManifestError::Io(e))?;

        toml::from_str(&content).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Сохранить манифест в файл.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ManifestError> {
        let content =
            toml::to_string_pretty(self).map_err(|e| ManifestError::Serialize(e.to_string()))?;

        fs::write(path.as_ref(), content).map_err(|e| ManifestError::Io(e))
    }

    /// Найти манифест в текущей директории или родительских.
    pub fn find() -> Option<std::path::PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let manifest_path = current.join(MANIFEST_FILE);
            if manifest_path.exists() {
                return Some(manifest_path);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Добавить зависимость.
    pub fn add_dependency(&mut self, name: &str, version: &str, dev: bool) {
        let dep = Dependency::Simple(version.to_string());

        if dev {
            self.dev_dependencies.insert(name.to_string(), dep);
        } else {
            self.dependencies.insert(name.to_string(), dep);
        }
    }

    /// Удалить зависимость.
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some() || self.dev_dependencies.remove(name).is_some()
    }
}

/// Ошибки работы с манифестом.
#[derive(Debug)]
pub enum ManifestError {
    Io(std::io::Error),
    Parse(String),
    Serialize(String),
    NotFound,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "IO error: {}", e),
            ManifestError::Parse(e) => write!(f, "Parse error: {}", e),
            ManifestError::Serialize(e) => write!(f, "Serialize error: {}", e),
            ManifestError::NotFound => write!(f, "Manifest file not found"),
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manifest() {
        let manifest = Manifest::new("test-project", false);
        assert_eq!(manifest.package.name, "test-project");
        assert_eq!(manifest.package.version, "0.1.0");
        assert_eq!(manifest.package.package_type, PackageType::Bin);
    }

    #[test]
    fn test_new_lib_manifest() {
        let manifest = Manifest::new("my-lib", true);
        assert_eq!(manifest.package.package_type, PackageType::Lib);
        assert_eq!(
            manifest.package.entry,
            Some("src/lib.syn".to_string())
        );
    }

    #[test]
    fn test_add_dependency() {
        let mut manifest = Manifest::new("test", false);
        manifest.add_dependency("math-utils", "1.0.0", false);

        assert!(manifest.dependencies.contains_key("math-utils"));
        assert_eq!(
            manifest.dependencies.get("math-utils").unwrap().version(),
            "1.0.0"
        );
    }

    #[test]
    fn test_serialize_manifest() {
        let manifest = Manifest::new("test", false);
        let toml = toml::to_string_pretty(&manifest).unwrap();

        assert!(toml.contains("[package]"));
        assert!(toml.contains("name = \"test\""));
    }
}
