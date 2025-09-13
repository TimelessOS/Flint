use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use comfy_table::Table;
use dialoguer::{Select, theme::ColorfulTheme};
#[cfg(feature = "network")]
use flintpkg::repo::network::add_repository;
use flintpkg::{
    build::{build, bundle::build_bundle},
    repo::{self, PackageManifest, get_package, read_manifest, remove_package, update_manifest},
    run::{install, quicklaunch::update_quicklaunch, start},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{get_quicklaunch_dir, get_repos_dir, system_data_dir, system_quicklaunch_dir},
    crypto::signing::sign,
};

mod config;
mod crypto;

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
            build(&build_manifest_path, &resolve_repo(base_path, &repo_name)?).await?;
        }

        Command::Install { repo_name, package } => {
            let target_repo_path: PathBuf = if let Some(repo_name) = repo_name {
                resolve_repo(base_path, &repo_name)?
            } else {
                resolve_package(base_path, &package, |_| true)?.0
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
                resolve_package(base_path, &id, |repo_path| {
                    repo_path.join("installed").join(&id).exists()
                })?
                .0
            };

            fs::remove_dir_all(target_repo_path.join("installed").join(&id))?;
        }

        Command::Bundle { command } => match command {
            BundleCommands::Extract => todo!(),
            BundleCommands::Create {
                repo_name,
                bundle_path,
                header_path,
            } => {
                let bundle = build_bundle(&header_path, &resolve_repo(base_path, &repo_name)?)?;
                fs::write(bundle_path, bundle)?;
            }
        },

        #[cfg(feature = "network")]
        Command::Update => {
            use flintpkg::run::quicklaunch::update_quicklaunch;

            update_all_repos(base_path).await?;

            update_quicklaunch(base_path, quicklaunch_bin_path)?;
        }

        Command::Run {
            repo_name,
            package,
            entrypoint,
            args,
        } => run_cmd(base_path, repo_name, package, entrypoint, args).await?,
    }

    Ok(())
}

#[cfg(feature = "network")]
async fn update_all_repos(base_path: &Path) -> Result<()> {
    use flintpkg::repo::{get_all_installed_packages, network::update_repository};

    for entry in base_path.read_dir()? {
        use flintpkg::repo::read_manifest;

        let repo = entry?;
        let repo_path = repo.path();
        let repo_name = repo.file_name();

        let update_changed_anything = update_repository(&repo_path).await?;

        if update_changed_anything {
            println!("Updating Repository '{}'", repo_name.display());
        } else {
            println!(
                "Skipping Repository '{}', no changes found",
                repo_name.display()
            );
        }

        let repo_manifest = read_manifest(&repo_path)?;

        for installed_package in get_all_installed_packages(&repo_path)? {
            let repo_package = get_package(&repo_manifest, &installed_package.id)?;

            if installed_package != repo_package {
                println!("Updating {}", repo_package.id);

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
        resolve_package(path, &package, |_| true)?.0
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

/// Resolve a repo name into a safe absolute path under the given base `path`.
fn resolve_repo(base: &Path, repo_name: &str) -> Result<PathBuf> {
    let candidate = base.join(repo_name);

    let base_canon = base
        .canonicalize()
        .context("Failed to canonicalize base path")?;
    let candidate_canon = candidate
        .canonicalize()
        .context("Failed to canonicalize repo path")?;

    if !candidate_canon.starts_with(&base_canon) {
        anyhow::bail!("Invalid repo path: escapes repository root");
    }

    Ok(candidate_canon)
}

/// Search all repositories for one matching a predicate
fn resolve_package<F>(
    path: &Path,
    package_id: &str,
    filter: F,
) -> Result<(PathBuf, PackageManifest)>
where
    F: Fn(&Path) -> bool,
{
    let mut possible_repos = Vec::new();

    for repo_entry in fs::read_dir(path)? {
        let repo_dir = repo_entry?;
        let repo_manifest = read_manifest(&repo_dir.path())?;

        let package = get_package(&repo_manifest, package_id);

        if let Ok(package) = package {
            let filtered = filter(&repo_dir.path());
            if filtered {
                possible_repos.push((repo_dir.path(), package));
            }
        }
    }

    if possible_repos.len() > 1 {
        return choose_repo(possible_repos);
    }

    if let Some(possible_repo) = possible_repos.first() {
        Ok(possible_repo.clone())
    } else {
        bail!("No Repositories contain that package.")
    }
}

fn choose_repo(
    possible_repos: Vec<(PathBuf, PackageManifest)>,
) -> Result<(PathBuf, PackageManifest)> {
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

    Ok(possible_repos.into_iter().nth(selection).unwrap())
}

async fn repo_commands(path: &Path, command: RepoCommands) -> Result<()> {
    match command {
        RepoCommands::Create { repo_name } => repo::create(&path.join(&repo_name))?,

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
                let repo_name_str = repo_name
                    .to_str()
                    .ok_or_else(|| anyhow!("Repository {} is not unicode.", repo_name.display()))?;

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

        #[cfg(feature = "network")]
        RepoCommands::Add {
            repo_name,
            remote_url,
        } => {
            let repo_path = &path.join(repo_name);
            fs::create_dir_all(repo_path)?;

            add_repository(repo_path, &remote_url, None).await?;
        }

        RepoCommands::Remove { repo_name } => {
            fs::remove_dir_all(resolve_repo(path, &repo_name)?)?;
        }

        RepoCommands::Update {
            homepage_url,
            license,
            title,
            version,
            repo_name,
            mirrors,
        } => {
            let repo_path = &resolve_repo(path, &repo_name)?;
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
            if let Some(mirrors) = mirrors {
                repo.mirrors = mirrors
                    .split(',')
                    .map(std::string::ToString::to_string)
                    .collect();
            }

            let manifest_serialized = &serde_yaml::to_string(&repo)?;
            let signature = sign(repo_path, manifest_serialized)?;

            update_manifest(repo_path, manifest_serialized, &signature.to_bytes())?;
        }

        RepoCommands::RemovePackage {
            repo_name,
            package_id,
        } => {
            remove_package(&package_id, &resolve_repo(path, &repo_name)?)?;
        }
    }

    Ok(())
}
