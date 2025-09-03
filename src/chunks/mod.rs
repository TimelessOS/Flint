mod tree;

pub use tree::*;

#[derive(serde::Deserialize, serde::Serialize)]
struct Chunk {
    /// Hash
    pub hash: String,

    /// Unix mode permissions
    permissions: u32,

    /// Expected size in kilobytes
    pub size: i64,
}
