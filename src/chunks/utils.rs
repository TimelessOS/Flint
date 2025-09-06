use std::{collections::HashSet, fs, path::Path};

use anyhow::Result;

use crate::{
    chunks::{Chunk, get_chunk_filename},
    repo::{get_all_installed_packages, get_all_packages},
};

/// Removes chunks that aren't actually used by any packages in the Repository
/// This is most useful for remote Repository administrators.
///
/// # Errors
///
/// - Filesystem errors (Permissions most likely)
/// - Repository doesn't exist
pub fn clean_unused(repo_path: &Path) -> Result<()> {
    let packages = get_all_packages(repo_path)?;
    let mut chunks: Vec<Chunk> = Vec::new();

    for package in packages {
        for chunk in package.chunks.clone() {
            chunks.push(chunk);
        }
    }

    clean(&repo_path.join("chunks"), &chunks)
}

/// Removes chunks that are actually used by any packages in the Repository, but aren't installed
/// This is most useful for end users.
///
/// # Errors
///
/// - Filesystem errors (Permissions most likely)
/// - Repository doesn't exist
pub fn clean_used(repo_path: &Path) -> Result<()> {
    let packages = get_all_installed_packages(repo_path)?;
    let mut chunks: Vec<Chunk> = Vec::new();

    for package in packages {
        for chunk in package.chunks.clone() {
            chunks.push(chunk);
        }
    }

    clean(&repo_path.join("chunks"), &chunks)
}

/// Cleans a `chunk_store` of unused chunks, using the whitelist `allowed_chunks`
fn clean(chunk_store_path: &Path, allowed_chunks: &[Chunk]) -> Result<()> {
    let allowed: HashSet<String> = allowed_chunks
        .iter()
        .map(|c| get_chunk_filename(&c.hash, c.permissions))
        .collect();

    for entry in fs::read_dir(chunk_store_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name_str) = file_name.to_str() else {
            continue;
        };

        if allowed.contains(file_name_str) {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}
