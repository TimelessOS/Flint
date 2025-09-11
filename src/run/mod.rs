use anyhow::{Context, Result, bail};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

#[cfg(feature = "network")]
use crate::chunks::install_tree;
use crate::{
    chunks::load_tree,
    repo::{self, read_manifest},
};

/// Starts a package from an entrypoint
///
/// # Errors
///
/// - Specified an entrypoint that doesn't exist
/// - Filesystem errors (Out of space, Permissions)
/// - Invalid Repository/Package manifest
pub fn start<S: AsRef<OsStr>>(
    repo_path: &Path,
    package_id: &str,
    entrypoint: &str,
    args: Vec<S>,
) -> Result<ExitStatus> {
    // This should use the `install.meta`, not the Repositories package
    let package_manifest =
        repo::get_package(repo_path, package_id).with_context(|| "Failed to get package")?;
    let installed_path = &repo_path.join("installed").join(package_id);

    // Get all matching commands
    let matches: Vec<&PathBuf> = package_manifest
        .commands
        .iter()
        .filter(|command| command.ends_with(entrypoint))
        .collect();

    // Make sure theres at least a single match
    if let Some(entrypoint) = matches.first() {
        // Install if not installed
        if !installed_path.exists() {
            install(repo_path, package_id).with_context(|| "Failed to install package.")?;
        }

        // Allow build_manifests to have a / at the start of entrypoints, eg: /bin/bash
        let entrypoint = entrypoint.to_string_lossy();
        let entrypoint: &str = entrypoint.trim_start_matches('/');

        // Actually run the command
        let status = Command::new(installed_path.join(entrypoint))
            .args(args)
            .status()?;

        Ok(status)
    } else {
        bail!("Entrypoint does not exist.")
    }
}

/// Installs or Updates a Package.
///
/// # Errors
///
/// - Filesystem errors (Out of space, Permissions)
/// - Invalid Repository/Package manifest
pub fn install(repo_path: &Path, package_id: &str) -> Result<()> {
    let package_manifest = repo::get_package(repo_path, package_id)
        .with_context(|| "Failed to get package from Repository.")?;
    let installed_path = &repo_path.join("installed").join(package_id);
    let repo_manifest = read_manifest(repo_path)?;

    #[cfg(feature = "network")]
    install_tree(
        &package_manifest.chunks,
        &repo_path.join("chunks"),
        &repo_manifest.mirrors,
        repo_manifest.hash_kind,
    )
    .with_context(|| "Failed to install package.")?;

    load_tree(
        installed_path,
        &repo_path.join("chunks"),
        &package_manifest.chunks,
    )
    .with_context(|| "Failed to rebuild the tree.")?;

    fs::write(
        installed_path.join("install.meta"),
        serde_yaml::to_string(&package_manifest)?,
    )?;

    Ok(())
}
