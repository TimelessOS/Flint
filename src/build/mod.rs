use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    chunks::save_tree,
    repo::{self, Metadata, PackageManifest, insert_package},
};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct BuildManifest {
    /// ID of this package, the main alias
    id: String,
    /// Aliases for installation
    #[serde(default)]
    aliases: Vec<String>,
    /// Package Metadata
    metadata: Metadata,
    /// A list of commands that this will give access to
    #[serde(default)]
    commands: Vec<String>,
    /// Directory relative to the manifest
    directory: PathBuf,
}

pub fn build(build_manifest_path: &Path, repo_path: &Path) -> Result<PackageManifest> {
    let build_manifest_path = &build_manifest_path.canonicalize()?;

    let build_manifest: BuildManifest =
        serde_yaml::from_str(&fs::read_to_string(build_manifest_path)?)?;

    let repo_manifest =
        repo::read_manifest(repo_path).with_context(|| "The target Repostiory does not exist")?;

    let build_manifest_parent = &build_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("/"));

    let chunks = save_tree(
        &build_manifest_parent.join(build_manifest.directory),
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

    insert_package(&package_manifest, repo_path)?;

    Ok(package_manifest)
}
