pub mod bundle;
mod sources;

use anyhow::{Context, Result, bail};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use temp_dir::TempDir;

use crate::{
    build::sources::get_sources,
    chunks::{load_tree, save_tree},
    repo::{self, Metadata, PackageManifest, get_package, insert_package, read_manifest},
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
    /// Script to be run after `build_script` but before packaging
    post_script: Option<PathBuf>,
    /// Sources to pull when building
    sources: Option<Vec<Source>>,
    /// ``SubPackages`` to be included directly into the output AND at build time.
    include: Option<Vec<String>>,
    /// ``SubPackages`` to be included directly ONLY at build time.
    /// Useful for SDKs.
    sdks: Option<Vec<String>>,
    /// RUNTIME environment variables
    env: Option<HashMap<String, String>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Source {
    /// Should either be git, tar or local
    kind: String,
    /// URL to the source.
    url: String,
    /// Path to extract.
    path: Option<String>,
    /// Git commit to use
    commit: Option<String>,
}

/// Builds and inserts a package into a Repository from a `build_manifest`
///
/// # Errors
///
/// - Filesystem (Out of Space, Permissions)
/// - Build Script Failure
pub async fn build(build_manifest_path: &Path, repo_path: &Path) -> Result<PackageManifest> {
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
        get_sources(build_dir.path(), build_manifest_parent, &sources).await?;
    }

    let mut envs = build_manifest.env.unwrap_or_default();

    if let Some(packages) = &build_manifest.include {
        for dependency in packages {
            let result = include(
                build_manifest_parent,
                dependency,
                build_dir.path(),
                repo_path,
            )?;

            envs.extend(result);
        }
    }

    if let Some(packages) = &build_manifest.sdks {
        for dependency in packages {
            let result = include(
                build_manifest_parent,
                dependency,
                build_dir.path(),
                repo_path,
            )?;

            envs.extend(result);
        }
    }

    if let Some(script) = build_manifest.build_script {
        let script_path = build_manifest_parent.join(script);

        let result = Command::new("sh")
            .arg("-c")
            .arg(script_path)
            .current_dir(build_dir.path())
            .status()?;

        if !result.success() {
            bail!("Build script failed.")
        }
    }

    let out_dir = build_dir.path().join(&build_manifest.directory);

    if let Some(script) = build_manifest.post_script {
        let script_path = build_manifest_parent.join(script);

        let result = Command::new("sh")
            .arg("-c")
            .arg(script_path)
            .current_dir(&out_dir)
            .status()?;

        if !result.success() {
            bail!("Build script failed.")
        }
    }

    let mut included_chunks = Vec::new();
    if let Some(packages) = &build_manifest.include {
        for dependency in packages {
            include(build_manifest_parent, dependency, &out_dir, repo_path)?;
        }
    }

    let chunks = save_tree(&out_dir, &repo_path.join("chunks"), repo_manifest.hash_kind)?;

    included_chunks.extend(chunks);

    let mut package_manifest = PackageManifest {
        aliases: build_manifest.aliases,
        commands: build_manifest.commands,
        id: build_manifest.id,
        metadata: build_manifest.metadata,
        chunks: included_chunks,
        env: None,
    };

    if !envs.is_empty() {
        package_manifest.env = Some(envs);
    }

    insert_package(&package_manifest, repo_path)?;

    Ok(package_manifest)
}

fn include(
    build_manifest_parent: &Path,
    dependency: &str,
    path_to_include_at: &Path,
    repo_path: &Path,
) -> Result<HashMap<String, String>> {
    let dependency_build_manifest_path = build_manifest_parent.join(dependency);
    let dependency_build_manifest: BuildManifest =
        serde_yaml::from_str(&fs::read_to_string(dependency_build_manifest_path)?)?;
    let repo_manifest = read_manifest(repo_path)?;
    let dependency_manifest = get_package(&repo_manifest, &dependency_build_manifest.id)?;

    load_tree(
        path_to_include_at,
        &repo_path.join("chunks"),
        &dependency_manifest.chunks,
    )?;

    Ok(dependency_manifest.env.unwrap_or_default())
}
