use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use comfy_table::Table;
use flint::{
    build::build,
    repo::{self, get_package, update_manifest},
    run::{install, start},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{crypto::signing::sign, utils::config::get_repos_dir};

mod crypto;
mod utils;

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
    /// Remove a package
    Remove {
        repo_name: Option<String>,
        package: String,
    },
    /// Interact with bundles
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },
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
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Creates a new Repository locally
    Create { repo_name: String },
    /// List all Repositories
    List,
    /// Add a Repository from a remote url
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

        repo_name: String,
    },
}

#[derive(Subcommand)]
enum BundleCommands {
    /// Extract a bundle into a Repository
    Extract,
    /// Extract a package from a Repository into a bundle
    Create,
}

#[derive(PartialEq)]
enum Scope {
    User,
    System,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let scope = if args.system {
        Scope::System
    } else if args.user {
        Scope::User
    } else {
        // default fallback, e.g. user
        Scope::User
    };

    let path = if scope == Scope::User {
        &get_repos_dir()?
    } else {
        Path::new("/var/lib/flint")
    };

    match args.command {
        Command::Repo { command } => repo_commands(path, command)?,

        Command::Build {
            build_manifest_path,
            repo_name,
        } => {
            build(&build_manifest_path, &path.join(repo_name))?;
        }

        Command::Install { repo_name, package } => {
            let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
                path.join(repo_name)
            } else {
                let mut possible_repos = Vec::new();

                for repo_entry in fs::read_dir(path)? {
                    let repo_dir = repo_entry?;
                    let package = get_package(&repo_dir.path(), &package);

                    if package.is_ok() {
                        possible_repos.push((repo_dir.path(), package?));
                    }
                }

                if possible_repos.is_empty() {
                    bail!("No Repositories contain that package.")
                }

                if possible_repos.len() == 1 {
                    possible_repos.first().unwrap().0.clone()
                } else {
                    todo!(
                        "Multiple Repositories contain Multiple versions of this package. Handling for this is currently not implemented."
                    )
                }
            };

            install(&target_repo_path, &package)?;
        }

        Command::Remove { repo_name, package } => todo!(),

        Command::Bundle { command } => todo!(),

        Command::Update => todo!(),

        Command::Run {
            repo_name,
            package,
            entrypoint,
            args,
        } => {
            let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
                path.join(repo_name)
            } else {
                let mut possible_repos = Vec::new();

                for repo_entry in fs::read_dir(path)? {
                    let repo_dir = repo_entry?;
                    let package = get_package(&repo_dir.path(), &package);

                    if package.is_ok() {
                        possible_repos.push((repo_dir.path(), package?));
                    }
                }

                if possible_repos.is_empty() {
                    bail!("No Repositories contain that package.")
                }

                if possible_repos.len() == 1 {
                    possible_repos.first().unwrap().0.clone()
                } else {
                    todo!(
                        "Multiple Repositories contain Multiple versions of this package. Handling for this is currently not implemented."
                    )
                }
            };

            let entrypoint = entrypoint.unwrap_or_else(|| {
                let package = get_package(&target_repo_path, &package).unwrap();

                // TODO: Make this cleaner
                package.commands.first().unwrap().to_str().unwrap().into()
            });

            start(
                &target_repo_path,
                &package,
                &entrypoint,
                args.unwrap_or_default(),
            )?;
        }
    };

    Ok(())
}

fn repo_commands(path: &Path, command: RepoCommands) -> Result<()> {
    match command {
        RepoCommands::Create { repo_name } => repo::create(&path.join(repo_name))?,

        RepoCommands::List => {
            let mut table = Table::new();

            table.set_header(vec![
                "Name",
                "Title",
                "URL",
                "Hash Kind",
                "Homepage",
                "License",
                "Version",
            ]);

            for repo_entry in fs::read_dir(path)? {
                let repo_dir = repo_entry?;
                let repo_name = repo_dir.file_name();
                let repo_name_str = repo_name.to_str().unwrap();

                let repo = repo::read_manifest(&repo_dir.path())?;

                table.add_row(vec![
                    &repo_name_str,
                    repo.metadata.title.unwrap_or_default().as_str(),
                    &repo.updates_url.unwrap_or_default(),
                    &repo.hash_kind.to_string(),
                    &repo.metadata.homepage_url.unwrap_or_default(),
                    &repo.metadata.license.unwrap_or_default(),
                    &repo.metadata.version.unwrap_or_default(),
                ]);
            }

            println!("{table}");
        }
        RepoCommands::Add {
            repo_name,
            remote_url,
        } => todo!(),

        RepoCommands::Remove { repo_name } => {
            fs::remove_dir_all(path.join(repo_name))?;
        }

        RepoCommands::Update {
            homepage_url,
            license,
            title,
            version,
            repo_name,
        } => {
            let repo_path = &path.join(repo_name);
            let mut repo = repo::read_manifest(repo_path)?;

            if title.is_some() {
                repo.metadata.title = title;
            }
            if homepage_url.is_some() {
                repo.metadata.homepage_url = homepage_url;
            }
            if license.is_some() {
                repo.metadata.license = license;
            }
            if version.is_some() {
                repo.metadata.version = version;
            }

            let manifest_serialized = &serde_yaml::to_string(&repo)?;
            let signature = sign(repo_path, manifest_serialized)?;

            update_manifest(repo_path, manifest_serialized, &signature.to_bytes())?;
        }
    };

    Ok(())
}
