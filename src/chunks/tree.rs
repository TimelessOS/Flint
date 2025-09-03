use std::path::Path;

use anyhow::Result;

use crate::chunks::Chunk;

pub fn save_tree(load_path: &Path, chunk_store_path: &Path) -> Result<Vec<Chunk>> {
    Ok(Vec::new())
}

pub fn load_tree(load_path: &Path, chunk_store_path: &Path, chunks: &Vec<Chunk>) -> Result<()> {
    Ok(())
}
