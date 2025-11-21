use anyhow::{Context, Result};
use std::{fs, os::unix::fs::symlink, path::Path};

use crate::{
    chunks::{HashKind, hash::hash, load_tree},
    repo::{PackageManifest, get_package, read_manifest},
};

fn hash_package(package_manifest: &PackageManifest, hash_kind: HashKind) -> Result<String> {
    let hash_str = serde_yaml::to_string(package_manifest)?;

    Ok(hash(hash_kind, hash_str.as_bytes()))
}

/// Installs the latest version of a package, assumes all chunks are available.
/// It is recommended you call `autoclean_versions` after.
///
/// # Errors
///
/// - Filesystem errors (Out of space, Permissions)
/// - Invalid Repository/Package manifest
///
/// # Returns
///
/// Returns the hash of the installed package
pub fn install_version(
    repo_path: &Path,
    package_id: &str,
    chunk_store_path: &Path,
) -> Result<String> {
    let repo_manifest = read_manifest(repo_path)?;

    let package_manifest = get_package(&repo_manifest, package_id)
        .with_context(|| "Failed to get package from Repository.")?;
    let package_hash = hash_package(&package_manifest, repo_manifest.hash_kind)?;
    let installed_path = &repo_path
        .join("versions")
        .join(format!("{}-{}", package_manifest.id, package_hash));

    load_tree(installed_path, chunk_store_path, &package_manifest.chunks)
        .with_context(|| "Failed to rebuild the tree.")?;

    fs::write(
        installed_path.join("install.meta"),
        serde_yaml::to_string(&package_manifest)?,
    )?;

    Ok(package_hash)
}

/// Switch to an older version/package hash.
///
/// # Errors
///
/// - Filesystem error during symlink (Within repo directory)
pub fn switch_version(repo_path: &Path, hash: &str, package_id: &str) -> Result<()> {
    let target_parent_path = repo_path.join("installed");
    let target_path = target_parent_path.join(package_id);
    let target_tmp_path = target_parent_path.join(format!("{package_id}.tmp"));
    fs::create_dir_all(target_parent_path)?;

    let versions_path = format!("../versions/{package_id}-{hash}");

    symlink(&versions_path, &target_tmp_path)?;
    fs::rename(&target_tmp_path, &target_path)?;

    Ok(())
}

/// Gets all versions for the `package_id`
///
/// # Errors
///
/// - Filesystem Read Errors (Permissions, etc)
///
/// # Returns
///
/// A `vec` containing `String`s of all version hashes
pub fn get_versions(repo_path: &Path, package_id: &str) -> Result<Vec<String>> {
    let mut versions = Vec::new();

    for entry in repo_path.join("versions").read_dir()? {
        if let Ok(entry) = entry
            && let Ok(file_name) = entry.file_name().into_string()
        {
            let split: Vec<&str> = file_name.split('-').collect();

            if let Some((version_hash, file_name_split)) = split.split_last()
                && file_name_split.join("-") == package_id
            {
                versions.push((*version_hash).to_string());
            }
        }
    }

    Ok(versions)
}

/// Removes a version of a package.
///
/// # Errors
///
/// - Version is not installed for the package
/// - Filesystem Write Errors (Permissions, etc)
pub fn remove_version(repo_path: &Path, hash: &str, package_id: &str) -> Result<()> {
    let path = repo_path.join(format!("versions/{package_id}-{hash}"));

    if path.exists() {
        fs::remove_dir_all(path)?;
        Ok(())
    } else {
        anyhow::bail!("The version {hash} is not installed for package {package_id}")
    }
}
