use anyhow::{Result, bail};
use std::{fs, path::Path};
use walkdir::WalkDir;

use crate::{
    bundle::pad_header,
    repo::{get_installed_package, read_manifest},
};

/// The Repository should ONLY have 1 package.
///
/// # Errors
///
/// - More/Less than one package found
/// - Filesystem read errors
pub fn build_bundle(header_path: &Path, repo_path: &Path) -> Result<Vec<u8>> {
    let header = fs::read(header_path)?;
    let mut header = pad_header(header)?;

    let manifest = read_manifest(repo_path)?;

    if let Some(package) = manifest.packages.first() {
        if manifest.packages.len() != 1 {
            bail!("More than one package found.")
        }

        let _ = get_installed_package(repo_path, &package.id)?;

        let mut tar = compress(repo_path)?;
        header.append(&mut tar);

        Ok(header)
    } else {
        bail!("No packages found in Repository.");
    }
}

fn compress(repo_path: &Path) -> Result<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());

    for entry in WalkDir::new(repo_path).min_depth(1) {
        let file = entry?;
        let path = file.path();

        if path.is_file() {
            // strip the repository root so the tar paths aren’t absolute
            let relative_path = path.strip_prefix(repo_path).unwrap();
            tar.append_path_with_name(path, relative_path)?;
        }
    }

    tar.finish()?;
    Ok(tar.into_inner()?)
}
