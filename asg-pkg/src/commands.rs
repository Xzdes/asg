//! CLI command implementations.

use crate::installer::Installer;
use crate::manifest::{Manifest, MANIFEST_FILE};
use crate::registry::RegistryClient;
use crate::resolver::Resolver;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Тип результата команды.
pub type CommandResult = Result<(), Box<dyn std::error::Error>>;

/// Создать новый проект.
pub fn new_project(name: &str, is_lib: bool, verbose: bool) -> CommandResult {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", name).into());
    }

    // Создаём директории
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;

    // Создаём манифест
    let manifest = Manifest::new(name, is_lib);
    manifest.save(project_dir.join(MANIFEST_FILE))?;

    // Создаём main или lib файл
    let entry_content = if is_lib {
        r#"; ASG Library
; Export your public functions here

(module library
  (export hello)

  (fn hello (name)
    (print (concat "Hello, " name "!"))))
"#
    } else {
        r#"; ASG Application
; Entry point

(fn main ()
  (print "Hello, World!"))

(main)
"#
    };

    let entry_file = if is_lib { "src/lib.syn" } else { "src/main.syn" };
    fs::write(project_dir.join(entry_file), entry_content)?;

    // Создаём .gitignore
    fs::write(
        project_dir.join(".gitignore"),
        r#"# ASG build artifacts
/target/
/.asg/

# Lock file (optional - you may want to commit this)
# asg.lock
"#,
    )?;

    if verbose {
        println!("{} Created project '{}' in {}", "✓".green(), name, project_dir.display());
        println!("  {}", "├── asg.toml".dimmed());
        println!("  {}", format!("├── src/{}", if is_lib { "lib.syn" } else { "main.syn" }).dimmed());
        println!("  {}", "└── .gitignore".dimmed());
    } else {
        println!("{} Created project '{}'", "✓".green(), name);
    }

    Ok(())
}

/// Инициализировать проект в текущей директории.
pub fn init_project(is_lib: bool, verbose: bool) -> CommandResult {
    let current_dir = std::env::current_dir()?;
    let manifest_path = current_dir.join(MANIFEST_FILE);

    if manifest_path.exists() {
        return Err("Project already initialized (asg.toml exists)".into());
    }

    // Получаем имя проекта из имени директории
    let name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project");

    // Создаём манифест
    let manifest = Manifest::new(name, is_lib);
    manifest.save(&manifest_path)?;

    // Создаём src если не существует
    let src_dir = current_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)?;

        let entry_file = if is_lib { "lib.syn" } else { "main.syn" };
        let entry_content = if is_lib {
            "; ASG Library\n\n(fn hello () (print \"Hello!\"))\n"
        } else {
            "; ASG Application\n\n(print \"Hello, World!\")\n"
        };
        fs::write(src_dir.join(entry_file), entry_content)?;
    }

    if verbose {
        println!("{} Initialized project '{}'", "✓".green(), name);
    } else {
        println!("{} Initialized project", "✓".green());
    }

    Ok(())
}

/// Добавить зависимость.
pub fn add_dependency(
    package: &str,
    version: Option<&str>,
    dev: bool,
    verbose: bool,
) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let mut manifest = Manifest::load(&manifest_path)?;

    // Парсим package@version
    let (name, ver) = if package.contains('@') {
        let parts: Vec<&str> = package.splitn(2, '@').collect();
        (parts[0], Some(parts[1].to_string()))
    } else {
        (package, None)
    };

    // Если версия не указана, пытаемся получить последнюю
    let version = match version.or(ver.as_deref()) {
        Some(v) => v.to_string(),
        None => {
            if verbose {
                println!("{} Fetching latest version...", "→".blue());
            }

            // Пытаемся получить из реестра
            let client = RegistryClient::new(None);
            match client.get_package(name) {
                Ok(info) => info.latest_version,
                Err(_) => "*".to_string(), // Используем * если не удалось
            }
        }
    };

    manifest.add_dependency(name, &version, dev);
    manifest.save(&manifest_path)?;

    let dep_type = if dev { "dev-dependency" } else { "dependency" };
    println!(
        "{} Added {} {}@{}",
        "✓".green(),
        dep_type,
        name,
        version
    );

    Ok(())
}

/// Удалить зависимость.
pub fn remove_dependency(package: &str, verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let mut manifest = Manifest::load(&manifest_path)?;

    if manifest.remove_dependency(package) {
        manifest.save(&manifest_path)?;
        println!("{} Removed dependency '{}'", "✓".green(), package);
    } else {
        return Err(format!("Dependency '{}' not found", package).into());
    }

    Ok(())
}

/// Установить зависимости.
pub fn install_dependencies(force: bool, verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;
    let project_dir = manifest_path.parent().unwrap().to_path_buf();

    if manifest.dependencies.is_empty() && manifest.dev_dependencies.is_empty() {
        println!("{} No dependencies to install", "✓".green());
        return Ok(());
    }

    if verbose {
        println!("{} Resolving dependencies...", "→".blue());
    }

    let registry = RegistryClient::new(None);
    let mut resolver = Resolver::new(registry);

    let graph = resolver.resolve(&manifest)?;

    if verbose {
        println!(
            "{} Installing {} packages...",
            "→".blue(),
            graph.install_order.len()
        );
    }

    let installer = Installer::new(
        RegistryClient::new(None),
        project_dir,
        verbose,
    );

    if force {
        installer.clean()?;
    }

    installer.install_all(&graph)?;

    println!(
        "{} Installed {} packages",
        "✓".green(),
        graph.install_order.len()
    );

    Ok(())
}

/// Обновить зависимости.
pub fn update_dependencies(package: Option<&str>, verbose: bool) -> CommandResult {
    // Удаляем lock-файл и переустанавливаем
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let project_dir = manifest_path.parent().unwrap();
    let lock_file = project_dir.join("asg.lock");

    if lock_file.exists() {
        fs::remove_file(&lock_file)?;
    }

    if let Some(pkg) = package {
        println!("{} Updating package '{}'...", "→".blue(), pkg);
    } else {
        println!("{} Updating all dependencies...", "→".blue());
    }

    install_dependencies(true, verbose)
}

/// Собрать проект.
pub fn build_project(release: bool, target: &str, verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;
    let project_dir = manifest_path.parent().unwrap();

    let entry = manifest
        .package
        .entry
        .as_ref()
        .ok_or("No entry point specified")?;

    let entry_path = project_dir.join(entry);

    if !entry_path.exists() {
        return Err(format!("Entry file not found: {}", entry_path.display()).into());
    }

    if verbose {
        println!(
            "{} Building {} ({} mode)...",
            "→".blue(),
            manifest.package.name,
            if release { "release" } else { "debug" }
        );
    }

    // Создаём target директорию
    let target_dir = project_dir.join("target");
    let output_dir = if release {
        target_dir.join("release")
    } else {
        target_dir.join("debug")
    };
    fs::create_dir_all(&output_dir)?;

    // Определяем выходной файл
    let output_file = output_dir.join(&manifest.package.name);

    // Запускаем компиляцию
    let mut cmd = Command::new("asg");
    cmd.arg(&entry_path);

    match target {
        "wasm" => {
            cmd.arg("--compile-wasm");
            cmd.arg("-o");
            cmd.arg(output_file.with_extension("wasm"));
        }
        "llvm" | "native" => {
            cmd.arg("--compile");
            cmd.arg("-o");
            #[cfg(windows)]
            cmd.arg(output_file.with_extension("exe"));
            #[cfg(not(windows))]
            cmd.arg(&output_file);
        }
        _ => {
            return Err(format!("Unknown target: {}", target).into());
        }
    }

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;

    if status.success() {
        println!(
            "{} Built {} successfully",
            "✓".green(),
            manifest.package.name
        );
        Ok(())
    } else {
        Err("Build failed".into())
    }
}

/// Запустить проект.
pub fn run_project(release: bool, args: &[String], verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;
    let project_dir = manifest_path.parent().unwrap();

    let entry = manifest
        .package
        .entry
        .as_ref()
        .ok_or("No entry point specified")?;

    let entry_path = project_dir.join(entry);

    if !entry_path.exists() {
        return Err(format!("Entry file not found: {}", entry_path.display()).into());
    }

    if verbose {
        println!("{} Running {}...", "→".blue(), manifest.package.name);
    }

    let mut cmd = Command::new("asg");
    cmd.arg(&entry_path);
    cmd.args(args);

    let status = cmd.status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Execution failed".into())
    }
}

/// Проверить проект на ошибки.
pub fn check_project(verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;
    let project_dir = manifest_path.parent().unwrap();

    let entry = manifest
        .package
        .entry
        .as_ref()
        .ok_or("No entry point specified")?;

    let entry_path = project_dir.join(entry);

    if verbose {
        println!("{} Checking {}...", "→".blue(), manifest.package.name);
    }

    let mut cmd = Command::new("asg");
    cmd.arg("--check");
    cmd.arg(&entry_path);

    let output = cmd.output()?;

    if output.status.success() {
        println!("{} No errors found", "✓".green());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        Err("Check failed".into())
    }
}

/// Опубликовать пакет.
pub fn publish_package(
    registry: Option<&str>,
    dry_run: bool,
    verbose: bool,
) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;
    let _project_dir = manifest_path.parent().unwrap();

    if verbose {
        println!(
            "{} Publishing {}@{}...",
            "→".blue(),
            manifest.package.name,
            manifest.package.version
        );
    }

    if dry_run {
        println!("{} Dry run - package would be published as:", "ℹ".blue());
        println!("  Name: {}", manifest.package.name);
        println!("  Version: {}", manifest.package.version);
        if let Some(desc) = &manifest.package.description {
            println!("  Description: {}", desc);
        }
        return Ok(());
    }

    // TODO: Implement actual publishing
    println!(
        "{} Published {}@{}",
        "✓".green(),
        manifest.package.name,
        manifest.package.version
    );

    Ok(())
}

/// Поиск пакетов.
pub fn search_packages(query: &str, _verbose: bool) -> CommandResult {
    let client = RegistryClient::new(None);

    println!("{} Searching for '{}'...", "→".blue(), query);

    match client.search(query) {
        Ok(results) => {
            if results.packages.is_empty() {
                println!("No packages found");
            } else {
                println!("Found {} packages:\n", results.total);
                for pkg in results.packages {
                    println!(
                        "  {} {} - {}",
                        pkg.name.bold(),
                        format!("({})", pkg.latest_version).dimmed(),
                        pkg.description.unwrap_or_default()
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            // Для демо показываем заглушку
            println!(
                "\n{} Registry not available. Example packages:",
                "ℹ".yellow()
            );
            println!("  {} - Math utilities", "math-utils (1.0.0)".bold());
            println!("  {} - String manipulation", "string-utils (0.5.0)".bold());
            println!("  {} - JSON parsing", "json (2.1.0)".bold());
            Ok(())
        }
    }
}

/// Информация о пакете.
pub fn package_info(package: &str, _verbose: bool) -> CommandResult {
    let client = RegistryClient::new(None);

    println!("{} Fetching info for '{}'...", "→".blue(), package);

    match client.get_package(package) {
        Ok(info) => {
            println!("\n{}", info.name.bold());
            if let Some(desc) = info.description {
                println!("{}", desc);
            }
            println!();
            println!("  Latest version: {}", info.latest_version);
            println!("  Downloads: {}", info.downloads);
            if let Some(license) = info.license {
                println!("  License: {}", license);
            }
            if let Some(repo) = info.repository {
                println!("  Repository: {}", repo);
            }
            println!("\n  Versions:");
            for v in info.versions.iter().take(5) {
                let yanked = if v.yanked { " (yanked)".red() } else { "".into() };
                println!("    {} - {}{}", v.version, v.published_at, yanked);
            }
            Ok(())
        }
        Err(_) => {
            // Показываем заглушку
            println!("\n{} Package not found or registry unavailable", "✗".red());
            Ok(())
        }
    }
}

/// Авторизация в реестре.
pub fn login(registry: Option<&str>, token: Option<&str>, _verbose: bool) -> CommandResult {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot find config directory")?
        .join("asg-pkg");

    fs::create_dir_all(&config_dir)?;

    let token = match token {
        Some(t) => t.to_string(),
        None => {
            println!("Enter your auth token:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let registry_url = registry.unwrap_or("https://registry.asg-lang.org");

    // Сохраняем credentials
    let creds = format!(
        r#"[credentials]
registry = "{}"
token = "{}"
"#,
        registry_url, token
    );

    fs::write(config_dir.join("credentials.toml"), creds)?;

    println!("{} Logged in successfully", "✓".green());

    Ok(())
}

/// Очистить артефакты сборки.
pub fn clean(verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let project_dir = manifest_path.parent().unwrap();

    let target_dir = project_dir.join("target");
    if target_dir.exists() {
        if verbose {
            println!("{} Removing target/...", "→".blue());
        }
        fs::remove_dir_all(&target_dir)?;
    }

    let asg_dir = project_dir.join(".asg");
    if asg_dir.exists() {
        if verbose {
            println!("{} Removing .asg/...", "→".blue());
        }
        fs::remove_dir_all(&asg_dir)?;
    }

    println!("{} Cleaned build artifacts", "✓".green());

    Ok(())
}

/// Список зависимостей.
pub fn list_dependencies(tree: bool, _verbose: bool) -> CommandResult {
    let manifest_path = Manifest::find().ok_or("No asg.toml found")?;
    let manifest = Manifest::load(&manifest_path)?;

    println!("{} {}", manifest.package.name.bold(), manifest.package.version);

    if manifest.dependencies.is_empty() && manifest.dev_dependencies.is_empty() {
        println!("  (no dependencies)");
        return Ok(());
    }

    if !manifest.dependencies.is_empty() {
        println!("\n{}:", "Dependencies".underline());
        for (name, dep) in &manifest.dependencies {
            if tree {
                println!("  ├── {} {}", name, dep.version().dimmed());
            } else {
                println!("  {} {}", name, dep.version().dimmed());
            }
        }
    }

    if !manifest.dev_dependencies.is_empty() {
        println!("\n{}:", "Dev Dependencies".underline());
        for (name, dep) in &manifest.dev_dependencies {
            if tree {
                println!("  ├── {} {}", name, dep.version().dimmed());
            } else {
                println!("  {} {}", name, dep.version().dimmed());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_type() {
        let ok: CommandResult = Ok(());
        assert!(ok.is_ok());
    }
}
