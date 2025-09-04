use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    chunks::save_tree,
    crypto::signing::sign,
    repo::{self, Metadata, PackageManifest, update_manifest},
};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct BuildManifest {
    /// ID of this package, the main alias
    id: String,
    /// Aliases for installation
    aliases: Vec<String>,
    /// Package Metadata
    metadata: Metadata,
    /// A list of commands that this will give access to
    commands: Vec<String>,
    /// Directory relative to the manifest
    directory: PathBuf,
}

pub fn build(build_manifest_path: &Path, repo_path: &Path) -> Result<PackageManifest> {
    let build_manifest: BuildManifest =
        serde_yaml::from_str(&fs::read_to_string(build_manifest_path)?)?;

    let repo_manifest = repo::read_manifest_unsigned(repo_path)?;

    let chunks = save_tree(
        &build_manifest_path.join(build_manifest.directory),
        &repo_path.join("chunks"),
        repo_manifest.hash_kind,
    )?;

    let package_manifest = PackageManifest {
        aliases: build_manifest.aliases,
        commands: build_manifest.commands,
        id: build_manifest.id,
        metadata: build_manifest.metadata,
        chunks,
    };

    let package_manifest_serialized = &serde_yaml::to_string(&package_manifest)?;

    update_manifest(
        repo_path,
        package_manifest_serialized,
        &sign(repo_path, package_manifest_serialized)?.to_bytes(),
    )?;

    Ok(package_manifest)
}
