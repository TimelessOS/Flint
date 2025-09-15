use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::repo::{PackageManifest, get_package, read_manifest};

/// Resolve a repo name into a safe absolute path under the given base `path`.
///
/// # Errors
///
/// - Path cannot be canonicalized (Perhaps doesn't exist on disk?)
/// - Path is dangerous and escapes `base`
pub fn resolve_repo(base: &Path, repo_name: &str) -> Result<PathBuf> {
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
///
/// # Errors
///
/// - A Repository contains invalid data/signature
/// - Filesystem errors
pub fn resolve_package<F>(
    path: &Path,
    package_id: &str,
    filter: F,
) -> Result<Vec<(PathBuf, PackageManifest)>>
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

    Ok(possible_repos)
}
