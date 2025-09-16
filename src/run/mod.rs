pub mod quicklaunch;

use anyhow::{Context, Result, bail};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

#[cfg(feature = "network")]
use crate::chunks::install_tree;
use crate::{
    chunks::load_tree,
    repo::{self, PackageManifest, read_manifest},
};

/// Starts a package from an entrypoint
///
/// # Errors
///
/// - Specified an entrypoint that doesn't exist
/// - Filesystem errors (Out of space, Permissions)
/// - Invalid Repository/Package manifest
/// - Package is not installed
pub fn start<S: AsRef<OsStr>>(
    repo_path: &Path,
    package_manifest: PackageManifest,
    entrypoint: &str,
    args: Vec<S>,
) -> Result<ExitStatus> {
    let installed_path = &repo_path.join("installed").join(package_manifest.id);

    // Get all matching commands
    let matches: Vec<&PathBuf> = package_manifest
        .commands
        .iter()
        .filter(|command| command.ends_with(entrypoint))
        .collect();

    // Make sure theres at least a single match
    if let Some(entrypoint) = matches.first() {
        // Allow build_manifests to have a / at the start of entrypoints, eg: /bin/bash
        let entrypoint = entrypoint.to_string_lossy();
        let entrypoint: &str = entrypoint.trim_start_matches('/');

        let mut envs: HashMap<String, String> = package_manifest.env.unwrap_or_default();

        // I hate I have to do this.
        let keys_to_update: Vec<String> = envs
            .iter()
            .filter(|(_, v)| v.contains("./"))
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_update {
            if let Some(value) = envs.get_mut(&key) {
                *value = value.replace("./", &format!("{}/", &installed_path.to_string_lossy()));
            }
        }

        // Actually run the command
        let status = Command::new(installed_path.join(entrypoint))
            .args(args)
            .envs(envs)
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
pub async fn install(repo_path: &Path, package_id: &str) -> Result<()> {
    let repo_manifest = read_manifest(repo_path)?;

    let package_manifest = repo::get_package(&repo_manifest, package_id)
        .with_context(|| "Failed to get package from Repository.")?;
    let installed_path = &repo_path.join("installed").join(package_id);

    // Get any chunks that are not installed
    #[cfg(feature = "network")]
    install_tree(
        &package_manifest.chunks,
        &repo_path.join("chunks"),
        &repo_manifest.mirrors,
        repo_manifest.hash_kind,
    )
    .await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::save_tree;
    use crate::repo::{Metadata, PackageManifest, create, insert_package};
    use std::fs;
    use temp_dir::TempDir;

    #[tokio::test]
    async fn test_install() -> Result<()> {
        let repo_dir = TempDir::new()?;
        let repo_path = repo_dir.path();

        create(repo_path, Some(repo_path))?;

        // Create a temp tree
        let temp_tree = TempDir::new()?;
        fs::write(temp_tree.path().join("file1"), "content1")?;
        fs::create_dir(temp_tree.path().join("dir"))?;
        fs::write(temp_tree.path().join("dir/file2"), "content2")?;
        let chunks = save_tree(
            temp_tree.path(),
            &repo_path.join("chunks"),
            crate::chunks::HashKind::Blake3,
        )?;

        let package = PackageManifest {
            id: "testpkg".to_string(),
            aliases: vec![],
            metadata: Metadata {
                title: Some("Test".to_string()),
                description: None,
                homepage_url: None,
                version: None,
                license: None,
            },
            chunks,
            commands: Vec::new(),
            env: None,
        };

        // Insert package
        insert_package(&package, repo_path, Some(repo_path))?;

        // Now install
        install(repo_path, "testpkg").await?;

        // Check installed
        let installed_path = repo_path.join("installed/testpkg");
        assert!(installed_path.exists());
        assert!(installed_path.join("file1").exists());
        assert!(installed_path.join("dir/file2").exists());
        assert!(installed_path.join("install.meta").exists());

        Ok(())
    }
}
