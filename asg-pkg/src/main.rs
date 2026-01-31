//! ASG Package Manager
//!
//! Менеджер пакетов для языка ASG.
//!
//! # Использование
//!
//! ```bash
//! # Создать новый проект
//! asg-pkg new my-project
//!
//! # Инициализировать в существующей директории
//! asg-pkg init
//!
//! # Добавить зависимость
//! asg-pkg add math-utils
//! asg-pkg add math-utils@1.2.0
//!
//! # Установить все зависимости
//! asg-pkg install
//!
//! # Обновить зависимости
//! asg-pkg update
//!
//! # Собрать проект
//! asg-pkg build
//!
//! # Опубликовать пакет
//! asg-pkg publish
//! ```

mod manifest;
mod registry;
mod resolver;
mod installer;
mod commands;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::ExitCode;

/// ASG Package Manager
#[derive(Parser)]
#[command(name = "asg-pkg")]
#[command(author = "Pavel (Xzdes)")]
#[command(version = "0.1.0")]
#[command(about = "Package manager for ASG programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ASG project
    New {
        /// Project name
        name: String,

        /// Create a library instead of binary
        #[arg(long)]
        lib: bool,
    },

    /// Initialize ASG project in current directory
    Init {
        /// Create a library instead of binary
        #[arg(long)]
        lib: bool,
    },

    /// Add a dependency
    Add {
        /// Package name (optionally with version: package@1.0.0)
        package: String,

        /// Add as dev dependency
        #[arg(long)]
        dev: bool,

        /// Specific version
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        package: String,
    },

    /// Install dependencies
    Install {
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },

    /// Update dependencies
    Update {
        /// Specific package to update
        package: Option<String>,
    },

    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Target (native, wasm, llvm)
        #[arg(long, default_value = "native")]
        target: String,
    },

    /// Run the project
    Run {
        /// Run in release mode
        #[arg(long)]
        release: bool,

        /// Arguments to pass
        args: Vec<String>,
    },

    /// Check the project for errors
    Check,

    /// Publish package to registry
    Publish {
        /// Registry URL
        #[arg(long)]
        registry: Option<String>,

        /// Dry run (don't actually publish)
        #[arg(long)]
        dry_run: bool,
    },

    /// Search for packages
    Search {
        /// Search query
        query: String,
    },

    /// Show package info
    Info {
        /// Package name
        package: String,
    },

    /// Login to registry
    Login {
        /// Registry URL
        #[arg(long)]
        registry: Option<String>,

        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },

    /// Clean build artifacts
    Clean,

    /// List dependencies
    List {
        /// Show tree view
        #[arg(long)]
        tree: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name, lib } => commands::new_project(&name, lib, cli.verbose),
        Commands::Init { lib } => commands::init_project(lib, cli.verbose),
        Commands::Add { package, dev, version } => {
            commands::add_dependency(&package, version.as_deref(), dev, cli.verbose)
        }
        Commands::Remove { package } => commands::remove_dependency(&package, cli.verbose),
        Commands::Install { force } => commands::install_dependencies(force, cli.verbose),
        Commands::Update { package } => {
            commands::update_dependencies(package.as_deref(), cli.verbose)
        }
        Commands::Build { release, target } => {
            commands::build_project(release, &target, cli.verbose)
        }
        Commands::Run { release, args } => commands::run_project(release, &args, cli.verbose),
        Commands::Check => commands::check_project(cli.verbose),
        Commands::Publish { registry, dry_run } => {
            commands::publish_package(registry.as_deref(), dry_run, cli.verbose)
        }
        Commands::Search { query } => commands::search_packages(&query, cli.verbose),
        Commands::Info { package } => commands::package_info(&package, cli.verbose),
        Commands::Login { registry, token } => {
            commands::login(registry.as_deref(), token.as_deref(), cli.verbose)
        }
        Commands::Clean => commands::clean(cli.verbose),
        Commands::List { tree } => commands::list_dependencies(tree, cli.verbose),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !cli.quiet {
                eprintln!("{}: {}", "error".red().bold(), e);
            }
            ExitCode::FAILURE
        }
    }
}
