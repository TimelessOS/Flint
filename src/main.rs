use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::Table;
use flint::repo::{self, update_manifest};
use std::{fs, path::Path};

use crate::{crypto::signing::sign, utils::config::get_repos_dir};

mod crypto;
mod utils;

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(styles = CLAP_STYLING)]
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
    Build,
    /// Install a package
    Install,
    /// Remove a package
    Remove,
    /// Interact with bundles
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },
    /// Updates a repository and its packages
    Update,
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
        Command::Repo { command } => match command {
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

                    let repo = repo::read_manifest_unsigned(&repo_dir.path())?;

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

            RepoCommands::Remove { repo_name } => todo!(),

            RepoCommands::Update {
                homepage_url,
                license,
                title,
                version,
                repo_name,
            } => {
                let repo_path = &path.join(repo_name);
                let mut repo = repo::read_manifest_unsigned(repo_path)?;

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
        },

        Command::Build => todo!(),

        Command::Install => todo!(),

        Command::Remove => todo!(),

        Command::Bundle { command } => todo!(),

        Command::Update => todo!(),
    };

    Ok(())
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);
