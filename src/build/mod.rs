mod sources;

use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use temp_dir::TempDir;

use crate::{
    build::sources::get_sources,
    chunks::save_tree,
    repo::{self, Metadata, PackageManifest, insert_package, remove_package},
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
    commands: Vec<PathBuf>,
    /// Directory relative to the manifest
    directory: PathBuf,
    /// Edition
    edition: String,
    /// Script to be run before packaging
    build_script: Option<PathBuf>,

    sources: Option<Vec<Source>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Source {
    /// Should either be git, tar or local
    kind: String,
    /// URL to the source.
    url: String,
    /// Path to extract.
    path: Option<String>,
}

/// Builds and inserts a package into a Repository from a `build_manifest`
///
/// # Errors
///
/// - Filesystem (Out of Space, Permissions)
/// - Build Script Failure
pub fn build(build_manifest_path: &Path, repo_path: &Path) -> Result<PackageManifest> {
    let build_dir = TempDir::new()?;
    let build_manifest_path = &build_manifest_path.canonicalize()?;

    let build_manifest: BuildManifest =
        serde_yaml::from_str(&fs::read_to_string(build_manifest_path)?)?;

    let repo_manifest =
        repo::read_manifest(repo_path).with_context(|| "The target Repostiory does not exist")?;

    let build_manifest_parent = &build_manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("/"));

    if let Some(sources) = build_manifest.sources {
        get_sources(build_dir.path(), build_manifest_parent, &sources)?;
    }

    if let Some(script) = build_manifest.build_script {
        let result = Command::new("sh")
            .arg("-c")
            .arg(build_manifest_parent.join(script))
            .current_dir(build_dir.path())
            .status()?;

        if !result.success() {
            bail!("Build script failed.")
        }
    }

    let chunks = save_tree(
        &build_dir.path().join(&build_manifest.directory),
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

    let _ = remove_package(&package_manifest.id, repo_path);
    insert_package(&package_manifest, repo_path)?;

    Ok(package_manifest)
}
