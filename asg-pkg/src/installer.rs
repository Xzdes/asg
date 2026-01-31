//! Package installer.

use crate::registry::RegistryClient;
use crate::resolver::{DependencyGraph, ResolvedDependency};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Директория для установленных пакетов.
pub const PACKAGES_DIR: &str = ".asg/packages";

/// Lock-файл.
pub const LOCK_FILE: &str = "asg.lock";

/// Установщик пакетов.
pub struct Installer {
    /// Клиент реестра
    registry: RegistryClient,
    /// Директория проекта
    project_dir: PathBuf,
    /// Verbose mode
    verbose: bool,
}

impl Installer {
    /// Создать новый установщик.
    pub fn new(registry: RegistryClient, project_dir: PathBuf, verbose: bool) -> Self {
        Self {
            registry,
            project_dir,
            verbose,
        }
    }

    /// Установить все зависимости из графа.
    pub fn install_all(&self, graph: &DependencyGraph) -> Result<(), InstallerError> {
        let packages_dir = self.project_dir.join(PACKAGES_DIR);
        fs::create_dir_all(&packages_dir).map_err(InstallerError::Io)?;

        let total = graph.install_order.len();
        let pb = if self.verbose {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(pb)
        } else {
            None
        };

        for (i, name) in graph.install_order.iter().enumerate() {
            if let Some(ref pb) = pb {
                pb.set_message(format!("Installing {}", name));
                pb.set_position(i as u64);
            }

            let dep = graph
                .resolved
                .get(name)
                .ok_or_else(|| InstallerError::PackageNotFound(name.clone()))?;

            self.install_package(dep, &packages_dir)?;
        }

        if let Some(pb) = pb {
            pb.finish_with_message("Done!");
        }

        // Создаём lock-файл
        self.write_lock_file(graph)?;

        Ok(())
    }

    /// Установить один пакет.
    fn install_package(
        &self,
        dep: &ResolvedDependency,
        packages_dir: &Path,
    ) -> Result<(), InstallerError> {
        let package_dir = packages_dir.join(&dep.name).join(&dep.version);

        // Проверяем, не установлен ли уже
        if package_dir.exists() {
            if self.verify_checksum(&package_dir, dep.checksum.as_deref())? {
                return Ok(()); // Уже установлен и проверен
            }
            // Удаляем повреждённый пакет
            fs::remove_dir_all(&package_dir).map_err(InstallerError::Io)?;
        }

        // Скачиваем пакет
        let package_data = self
            .registry
            .download(&dep.name, &dep.version)
            .map_err(|e| InstallerError::Download(e.to_string()))?;

        // Проверяем контрольную сумму
        if let Some(expected) = &dep.checksum {
            let actual = self.compute_checksum(&package_data);
            if &actual != expected {
                return Err(InstallerError::ChecksumMismatch {
                    expected: expected.clone(),
                    actual,
                });
            }
        }

        // Распаковываем
        fs::create_dir_all(&package_dir).map_err(InstallerError::Io)?;
        self.extract_package(&package_data, &package_dir)?;

        Ok(())
    }

    /// Распаковать пакет.
    fn extract_package(&self, data: &[u8], dest: &Path) -> Result<(), InstallerError> {
        let cursor = std::io::Cursor::new(data);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| InstallerError::Extract(e.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| InstallerError::Extract(e.to_string()))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if file.is_dir() {
                fs::create_dir_all(&outpath).map_err(InstallerError::Io)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(InstallerError::Io)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(InstallerError::Io)?;
                std::io::copy(&mut file, &mut outfile).map_err(InstallerError::Io)?;
            }

            // Set permissions on unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                        .map_err(InstallerError::Io)?;
                }
            }
        }

        Ok(())
    }

    /// Вычислить контрольную сумму.
    fn compute_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Проверить контрольную сумму установленного пакета.
    fn verify_checksum(
        &self,
        package_dir: &Path,
        expected: Option<&str>,
    ) -> Result<bool, InstallerError> {
        let checksum_file = package_dir.join(".checksum");

        if !checksum_file.exists() {
            return Ok(expected.is_none());
        }

        let actual = fs::read_to_string(&checksum_file).map_err(InstallerError::Io)?;

        match expected {
            Some(exp) => Ok(actual.trim() == exp),
            None => Ok(true),
        }
    }

    /// Записать lock-файл.
    fn write_lock_file(&self, graph: &DependencyGraph) -> Result<(), InstallerError> {
        let lock_path = self.project_dir.join(LOCK_FILE);

        let mut content = String::from("# ASG lock file - DO NOT EDIT\n");
        content.push_str("# This file is auto-generated by asg-pkg\n\n");

        for name in &graph.install_order {
            if let Some(dep) = graph.resolved.get(name) {
                content.push_str(&format!("[[package]]\n"));
                content.push_str(&format!("name = \"{}\"\n", dep.name));
                content.push_str(&format!("version = \"{}\"\n", dep.version));
                if let Some(ref checksum) = dep.checksum {
                    content.push_str(&format!("checksum = \"{}\"\n", checksum));
                }
                if !dep.dependencies.is_empty() {
                    content.push_str("dependencies = [\n");
                    for d in &dep.dependencies {
                        content.push_str(&format!("    \"{}\",\n", d));
                    }
                    content.push_str("]\n");
                }
                content.push('\n');
            }
        }

        fs::write(&lock_path, content).map_err(InstallerError::Io)?;

        Ok(())
    }

    /// Очистить установленные пакеты.
    pub fn clean(&self) -> Result<(), InstallerError> {
        let packages_dir = self.project_dir.join(PACKAGES_DIR);
        if packages_dir.exists() {
            fs::remove_dir_all(&packages_dir).map_err(InstallerError::Io)?;
        }

        let lock_file = self.project_dir.join(LOCK_FILE);
        if lock_file.exists() {
            fs::remove_file(&lock_file).map_err(InstallerError::Io)?;
        }

        Ok(())
    }
}

/// Ошибки установщика.
#[derive(Debug)]
pub enum InstallerError {
    Io(std::io::Error),
    Download(String),
    Extract(String),
    PackageNotFound(String),
    ChecksumMismatch { expected: String, actual: String },
}

impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallerError::Io(e) => write!(f, "IO error: {}", e),
            InstallerError::Download(e) => write!(f, "Download error: {}", e),
            InstallerError::Extract(e) => write!(f, "Extract error: {}", e),
            InstallerError::PackageNotFound(name) => write!(f, "Package not found: {}", name),
            InstallerError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for InstallerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_checksum() {
        let installer = Installer::new(
            RegistryClient::new(None),
            PathBuf::from("."),
            false,
        );

        let data = b"test data";
        let checksum = installer.compute_checksum(data);

        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA-256 = 64 hex chars
    }
}
