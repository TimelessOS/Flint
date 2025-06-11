#![warn(clippy::pedantic)]

use anyhow::Result;
use clap::Subcommand;
pub use flint_types::Metadata;
use fs_extra::dir::CopyOptions;
use lcas::build;
use std::path::PathBuf;
use std::{fs, path::Path};
use tempdir::TempDir;

#[derive(Subcommand, Debug)]
pub enum RepoCommands {
    /// Create a Package
    CreatePackage {
        /// Directory of the Repo
        repo: PathBuf,
        /// Directory of the Package to be Packaged
        dir: PathBuf,
        /// Name of the package
        name: String,
    },
    /// Create a Repo
    CreateRepo {
        /// Directory of the Repo
        dir: PathBuf,
    },
}

/// # Errors
/// This will error out if any of the subcommands fail. As this is generally going to be passed to the user, this should* be fine.
pub fn main(args: RepoCommands) -> Result<()> {
    match args {
        RepoCommands::CreatePackage { repo, dir, name } => {
            let metadata = Metadata {
                name,
                description: None,
                url: None,
                license: None,
                arch: None,
            };
            package(&metadata, &dir, &repo)
        }
        RepoCommands::CreateRepo { dir } => create_repo(&dir),
    }
}

/// Creates a package
/// # Errors
/// - If the temp directory cannot be created
/// - If the build directory cannot be copied to the temp directory
/// - If theres a general build error (see `flint::build`)
/// - If the repo isn't a valid repo
pub fn package(metadata: &Metadata, dir: &Path, repo_path: &Path) -> Result<()> {
    let temp_dir = TempDir::new("flint-package")?;

    let copyoptions = CopyOptions {
        overwrite: true,
        buffer_size: 64000,
        skip_exist: false,
        copy_inside: false,
        content_only: true,
        depth: 0,
    };

    fs_extra::dir::copy(dir, &temp_dir, &copyoptions)?;

    fs::write(
        temp_dir.path().join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )?;

    build(
        &temp_dir.path().to_path_buf(),
        repo_path,
        metadata.name.as_str(),
    )?;

    Ok(())
}

/// Creates a repo
///
/// # Arguments
/// `repo_path` - The base directory for the repo.
///
/// # Errors
/// - If the directory already exists
pub fn create_repo(repo_path: &Path) -> Result<()> {
    lcas::create_repo(repo_path)
}

#[cfg(test)]
mod test_packaging {
    use super::*;

    #[test]
    fn test_create_repo_simple() -> Result<()> {
        create_repo(
            &TempDir::new("flint-test-create_repo_simple")?
                .path()
                .join("test"),
        )
    }
}
