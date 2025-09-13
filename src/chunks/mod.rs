pub mod hash;
#[cfg(feature = "network")]
pub mod network;
mod tree;
pub mod utils;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub use hash::HashKind;
pub use tree::*;

use crate::repo::read_manifest;

/// Verify all chunks in a repository
///
/// # Errors
///
/// - Filesystem errors
/// - Invalid manifests
pub fn verify_all_chunks(repo_path: &Path) -> anyhow::Result<()> {
    let repo_manifest = read_manifest(repo_path)?;
    let mut all_chunks = HashSet::new();

    for package in repo_manifest.packages {
        for chunk in package.chunks {
            all_chunks.insert((chunk.hash.clone(), chunk.permissions));
        }
    }

    let chunk_store_path = repo_path.join("chunks");
    let mut verified = 0;
    let mut failed = 0;

    for (expected_hash, perms) in &all_chunks {
        let chunk_path = chunk_store_path.join(get_chunk_filename(expected_hash, *perms));
        if !chunk_path.exists() {
            eprintln!("Missing chunk: {expected_hash}");
            failed += 1;
            continue;
        }

        let contents = std::fs::read(&chunk_path)?;
        let computed_hash = hash::hash(repo_manifest.hash_kind, &contents);

        if computed_hash == *expected_hash {
            verified += 1;
        } else {
            eprintln!(
                "Hash mismatch for chunk: {expected_hash} (expected {expected_hash}, got {computed_hash})"
            );
            failed += 1;
        }
    }

    println!("Verified {verified} chunks, {failed} failed");

    if failed > 0 {
        anyhow::bail!("Some chunks failed verification");
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Path
    path: PathBuf,

    /// Hash
    hash: String,

    /// Unix mode permissions
    permissions: u32,

    /// Expected size in kilobytes, rounded.
    size: u64,
}

fn get_chunk_filename(hash: &str, permissions: u32) -> String {
    let mut new_hash = hash.to_string();

    new_hash.push_str(&permissions.to_string());

    new_hash
}
