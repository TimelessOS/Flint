mod hash;
mod tree;

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
