mod hash;
#[cfg(feature = "network")]
pub mod network;
mod tree;
pub mod utils;

use std::path::PathBuf;

pub use hash::HashKind;
pub use tree::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
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
