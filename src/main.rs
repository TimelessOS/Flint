mod build;
mod bundle;
mod chunks;
mod commands;
mod config;
mod crypto;
mod log;
mod repo;
mod run;
mod utils;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use dialoguer::{Select, theme::ColorfulTheme};
use std::{
    env::var_os,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    commands::{bundle::bundle_commands, repo::repo_commands},
    log::add_to_path_notice,
};
use flintpkg::{
    build::build,
    chunks::verify_all_chunks,
    config::{get_quicklaunch_dir, get_repos_dir, system_data_dir, system_quicklaunch_dir},
    repo::{PackageManifest, get_package, read_manifest},
    run::{install, quicklaunch::update_quicklaunch, start},
    utils::{resolve_package, resolve_repo},
};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Install system-wide (requires root)
    #[arg(long, conflicts_with = "user")]
    system: bool,

    /// Install for current user only
    #[arg(long, conflicts_with = "system")]
    user: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Interact with Repositories
    Repo {
        #[command(subcommand)]
        command: RepoCommands,
    },
    /// Builds a package from a local manifest and directory
    Build {
        build_manifest_path: PathBuf,
        repo_name: String,
    },
    /// Install a package
    Install {
        /// The Repository to install from
        #[arg(long)]
        repo_name: Option<String>,
        /// The package to install
        package: String,
    },
    /// Remove an installed package
    Remove {
        /// The Repository to remove from
        #[arg(long)]
        repo_name: Option<String>,
        package: String,
    },
    /// Interact with bundles
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },
    #[cfg(feature = "network")]
    /// Updates a repository and its packages
    Update,
    /// Run a package's entrypoint
    Run {
        /// The Repository the package is in
        #[arg(long)]
        repo_name: Option<String>,
        /// The package to install
        package: String,
        /// The entrypoint in question. Will default to the first entrypoint
        entrypoint: Option<String>,
        /// Extra arguments
        args: Option<Vec<String>>,
    },
    /// Verify all chunks in a repository
    VerifyChunks {
        /// The Repository to verify chunks for
        #[arg(long)]
        repo_name: String,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Creates a new Repository locally
    Create { repo_name: String },
    /// List all Repositories
    List,
    /// Add a Repository from a remote url
    #[cfg(feature = "network")]
    Add {
        repo_name: String,
        remote_url: String,
    },
    /// Remove a Repository
    Remove { repo_name: String },
    /// Update a Repositories Metadata
    Update {
        #[arg(long)]
        homepage_url: Option<String>,
        #[arg(long)]
        license: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        /// Comma seperated list of all mirrors
        mirrors: Option<String>,

        repo_name: String,
    },
    /// Remove a Package from this Repository
    RemovePackage {
        repo_name: String,
        package_id: String,
    },
}

#[derive(Subcommand)]
enum BundleCommands {
    /// Extract a bundle into a Repository
    Extract,
    /// Extract a package from a Repository into a bundle
    Create {
        repo_name: String,
        bundle_path: PathBuf,
        header_path: PathBuf,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Scope {
    User,
    System,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let scope = if args.system {
        Scope::System
    } else {
        Scope::User
    };

    let base_path = &if scope == Scope::User {
        get_repos_dir()?
    } else {
        system_data_dir()
    };

    let quicklaunch_bin_path = &if scope == Scope::User {
        get_quicklaunch_dir()?
    } else {
        system_quicklaunch_dir()
    };

    match args.command {
        Command::Repo { command } => {
            repo_commands(base_path, command).await?;
            update_quicklaunch(base_path, quicklaunch_bin_path)?;
        }

        Command::Build {
            build_manifest_path,
            repo_name,
        } => {
            build(
                &build_manifest_path,
                &resolve_repo(base_path, &repo_name)?,
                None,
            )
            .await?;
        }

        Command::Install { repo_name, package } => {
            let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
                resolve_repo(base_path, &repo_name)?
            } else {
                let possible_repos = resolve_package(base_path, &package, |_| true)?;

                if possible_repos.len() > 1 {
                    choose_repo(possible_repos)?
                } else if let Some(possible_repo) = possible_repos.first() {
                    possible_repo.0.clone()
                } else {
                    bail!("No Repositories contain that package.")
                }
            };

            install(&target_repo_path, &package).await?;
        }

        Command::Remove {
            repo_name,
            package: id,
        } => {
            let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
                resolve_repo(base_path, &repo_name)?
            } else {
                let possible_repos = resolve_package(base_path, &id, |repo_path| {
                    repo_path.join("installed").join(&id).exists()
                })?;

                if possible_repos.len() > 1 {
                    choose_repo(possible_repos)?
                } else if let Some(possible_repo) = possible_repos.first() {
                    possible_repo.0.clone()
                } else {
                    bail!("No Repositories contain that package.")
                }
            };

            fs::remove_dir_all(target_repo_path.join("installed").join(&id))?;
        }

        Command::Bundle { command } => bundle_commands(base_path, command)?,

        #[cfg(feature = "network")]
        Command::Update => {
            update_all_repos(base_path).await?;

            update_quicklaunch(base_path, quicklaunch_bin_path)?;
        }

        Command::Run {
            repo_name,
            package,
            entrypoint,
            args,
        } => run_cmd(base_path, repo_name, package, entrypoint, args).await?,

        Command::VerifyChunks { repo_name } => {
            let target_repo_path = resolve_repo(base_path, &repo_name)?;
            verify_all_chunks(&target_repo_path)?;
        }
    }

    if let Some(path) = var_os("PATH")
        && !path
            .to_string_lossy()
            .contains(&*quicklaunch_bin_path.to_string_lossy())
    {
        add_to_path_notice(quicklaunch_bin_path);
    }

    Ok(())
}

#[cfg(feature = "network")]
async fn update_all_repos(base_path: &Path) -> Result<()> {
    use crate::log::{skipped_update_repo, updated_package, updated_repo};
    use flintpkg::repo::{get_all_installed_packages, network::update_repository};

    for entry in base_path.read_dir()? {
        let repo = entry?;
        let repo_path = repo.path();
        let repo_name = repo.file_name();

        let update_changed_anything = update_repository(&repo_path).await?;

        if update_changed_anything {
            updated_repo(&repo_name);
        } else {
            skipped_update_repo(&repo_name);
        }

        let repo_manifest = read_manifest(&repo_path)?;

        for installed_package in get_all_installed_packages(&repo_path)? {
            let repo_package = get_package(&repo_manifest, &installed_package.id)?;

            if installed_package != repo_package {
                updated_package(&repo_package);

                install(&repo_path, &repo_package.id).await?;
            }
        }
    }

    Ok(())
}

async fn run_cmd(
    path: &Path,
    repo_name: Option<String>,
    package: String,
    entrypoint: Option<String>,
    args: Option<Vec<String>>,
) -> Result<()> {
    let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
        resolve_repo(path, &repo_name)?
    } else {
        let possible_repos = resolve_package(path, &package, |_| true)?;

        if possible_repos.len() > 1 {
            choose_repo(possible_repos)?
        } else if let Some(possible_repo) = possible_repos.first() {
            possible_repo.0.clone()
        } else {
            bail!("No Repositories contain that package.")
        }
    };

    let repo_manifest = read_manifest(&target_repo_path)?;

    let entrypoint = if let Some(e) = entrypoint {
        e
    } else {
        let package =
            get_package(&repo_manifest, &package).context("Failed to read package manifest")?;

        let first_command = package
            .commands
            .first()
            .context("Package has no commands defined")?;

        first_command
            .to_str()
            .context("First command path was not valid UTF-8")?
            .to_string()
    };

    // Install if not installed
    #[cfg(feature = "network")]
    if !target_repo_path
        .join("installed/")
        .join(&package)
        .join("install.meta")
        .exists()
    {
        install(&target_repo_path, &package)
            .await
            .with_context(|| "Failed to install package.")?;
    }

    start(
        &target_repo_path,
        &package,
        &entrypoint,
        args.unwrap_or_default(),
    )?;

    Ok(())
}

fn choose_repo(possible_repos: Vec<(PathBuf, PackageManifest)>) -> Result<PathBuf> {
    let items: Vec<String> = possible_repos
        .iter()
        .map(|(path, manifest)| {
            format!(
                "{} ({} {})",
                path.file_name().unwrap().to_string_lossy(),
                manifest.metadata.title.clone().unwrap_or_default(),
                manifest.metadata.version.clone().unwrap_or_default()
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Multiple repositories contain this package, pick one")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(possible_repos.into_iter().nth(selection).unwrap().0)
}
