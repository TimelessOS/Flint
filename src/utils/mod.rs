use anyhow::{Context, Result, bail};
use dialoguer::{Select, theme::ColorfulTheme};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::repo::{PackageManifest, get_package, read_manifest};

/// Resolve a repo name into a safe absolute path under the given base `path`.
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
pub fn resolve_package<F>(
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
