//! Package registry client.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// URL реестра по умолчанию.
pub const DEFAULT_REGISTRY: &str = "https://registry.asg-lang.org";

/// Клиент реестра пакетов.
pub struct RegistryClient {
    /// URL реестра
    pub registry_url: String,
    /// HTTP клиент
    client: reqwest::blocking::Client,
    /// Токен авторизации
    token: Option<String>,
}

/// Информация о пакете в реестре.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Имя пакета
    pub name: String,
    /// Описание
    pub description: Option<String>,
    /// Последняя версия
    pub latest_version: String,
    /// Все версии
    pub versions: Vec<VersionInfo>,
    /// Авторы
    pub authors: Vec<String>,
    /// Лицензия
    pub license: Option<String>,
    /// URL репозитория
    pub repository: Option<String>,
    /// Ключевые слова
    pub keywords: Vec<String>,
    /// Количество загрузок
    pub downloads: u64,
    /// Дата создания
    pub created_at: String,
    /// Дата обновления
    pub updated_at: String,
}

/// Информация о версии пакета.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Номер версии
    pub version: String,
    /// Дата публикации
    pub published_at: String,
    /// Контрольная сумма
    pub checksum: String,
    /// Зависимости
    pub dependencies: HashMap<String, String>,
    /// Минимальная версия ASG
    pub asg_version: Option<String>,
    /// Yanked (отозвана)
    pub yanked: bool,
}

/// Результат поиска.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Найденные пакеты
    pub packages: Vec<SearchHit>,
    /// Общее количество
    pub total: u64,
}

/// Результат поиска - один пакет.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// Имя пакета
    pub name: String,
    /// Описание
    pub description: Option<String>,
    /// Последняя версия
    pub latest_version: String,
    /// Количество загрузок
    pub downloads: u64,
}

impl RegistryClient {
    /// Создать клиент реестра.
    pub fn new(registry_url: Option<&str>) -> Self {
        Self {
            registry_url: registry_url
                .unwrap_or(DEFAULT_REGISTRY)
                .trim_end_matches('/')
                .to_string(),
            client: reqwest::blocking::Client::new(),
            token: None,
        }
    }

    /// Установить токен авторизации.
    pub fn set_token(&mut self, token: &str) {
        self.token = Some(token.to_string());
    }

    /// Получить информацию о пакете.
    pub fn get_package(&self, name: &str) -> Result<PackageInfo, RegistryError> {
        let url = format!("{}/api/v1/packages/{}", self.registry_url, name);

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().map_err(|e| RegistryError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .map_err(|e| RegistryError::Parse(e.to_string()))
        } else if response.status().as_u16() == 404 {
            Err(RegistryError::PackageNotFound(name.to_string()))
        } else {
            Err(RegistryError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )))
        }
    }

    /// Получить информацию о конкретной версии.
    pub fn get_version(&self, name: &str, version: &str) -> Result<VersionInfo, RegistryError> {
        let url = format!(
            "{}/api/v1/packages/{}/versions/{}",
            self.registry_url, name, version
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .map_err(|e| RegistryError::Parse(e.to_string()))
        } else if response.status().as_u16() == 404 {
            Err(RegistryError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ))
        } else {
            Err(RegistryError::Api(format!("HTTP {}", response.status())))
        }
    }

    /// Поиск пакетов.
    pub fn search(&self, query: &str) -> Result<SearchResult, RegistryError> {
        let url = format!("{}/api/v1/search?q={}", self.registry_url, query);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .json()
                .map_err(|e| RegistryError::Parse(e.to_string()))
        } else {
            Err(RegistryError::Api(format!("HTTP {}", response.status())))
        }
    }

    /// Скачать пакет.
    pub fn download(&self, name: &str, version: &str) -> Result<Vec<u8>, RegistryError> {
        let url = format!(
            "{}/api/v1/packages/{}/versions/{}/download",
            self.registry_url, name, version
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .bytes()
                .map(|b| b.to_vec())
                .map_err(|e| RegistryError::Network(e.to_string()))
        } else {
            Err(RegistryError::Api(format!("HTTP {}", response.status())))
        }
    }

    /// Опубликовать пакет.
    pub fn publish(&self, package_data: &[u8], name: &str) -> Result<(), RegistryError> {
        let token = self
            .token
            .as_ref()
            .ok_or(RegistryError::Unauthorized)?;

        let url = format!("{}/api/v1/packages/{}/publish", self.registry_url, name);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/octet-stream")
            .body(package_data.to_vec())
            .send()
            .map_err(|e| RegistryError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else if response.status().as_u16() == 401 {
            Err(RegistryError::Unauthorized)
        } else {
            Err(RegistryError::Api(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )))
        }
    }
}

/// Ошибки работы с реестром.
#[derive(Debug)]
pub enum RegistryError {
    Network(String),
    Parse(String),
    Api(String),
    PackageNotFound(String),
    VersionNotFound(String, String),
    Unauthorized,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Network(e) => write!(f, "Network error: {}", e),
            RegistryError::Parse(e) => write!(f, "Parse error: {}", e),
            RegistryError::Api(e) => write!(f, "API error: {}", e),
            RegistryError::PackageNotFound(name) => write!(f, "Package not found: {}", name),
            RegistryError::VersionNotFound(name, ver) => {
                write!(f, "Version {} not found for package {}", ver, name)
            }
            RegistryError::Unauthorized => write!(f, "Unauthorized. Please login first."),
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_creation() {
        let client = RegistryClient::new(None);
        assert_eq!(client.registry_url, DEFAULT_REGISTRY);
    }

    #[test]
    fn test_custom_registry() {
        let client = RegistryClient::new(Some("https://custom.registry.com/"));
        assert_eq!(client.registry_url, "https://custom.registry.com");
    }
}
