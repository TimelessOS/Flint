use anyhow::{Context, Result};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use walkdir::WalkDir;

use crate::chunks::{Chunk, HashKind, hash::hash};

pub fn save_tree(
    tree_path: &Path,
    chunk_store_path: &Path,
    hash_kind: &HashKind,
) -> Result<Vec<Chunk>> {
    let mut chunks = Vec::new();

    for entry in WalkDir::new(tree_path) {
        let file = entry?;

        if !file.file_type().is_file() {
            continue;
        }

        let path = file.path().strip_prefix(tree_path)?.to_path_buf();
        let contents = fs::read(file.path())?;
        let size = (contents.len() as u64) / 1024;
        let hash = hash(hash_kind, &contents);
        let mode = file.metadata()?.permissions().mode() & 0o777;

        // TODO: Make this hardlink if on the same filesystem.
        fs::write(
            chunk_store_path.join(get_chunk_filename(&hash, mode)),
            contents,
        )?;

        chunks.push(Chunk {
            hash,
            path,
            size,
            permissions: mode,
        });
    }

    Ok(chunks)
}

/// Turns a tree into a list of chunks
pub fn load_tree(load_path: &Path, chunk_store_path: &Path, chunks: &[Chunk]) -> Result<()> {
    for chunk in chunks {
        let extracted_path = load_path.join(&chunk.path);
        let chunk_path = chunk_store_path.join(get_chunk_filename(&chunk.hash, chunk.permissions));

        // Create parent path
        if let Some(parent) = extracted_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if fs::hard_link(&chunk_path, &extracted_path).is_err() {
            fs::copy(&chunk_path, &extracted_path)
                .with_context(|| "Could not copy data while extracting")?;
        };

        let mut perms = fs::metadata(&extracted_path)?.permissions();
        perms.set_mode(chunk.permissions & 0o777);
        fs::set_permissions(&extracted_path, perms)?;
    }

    Ok(())
}

/// Returns the tree's estimated size in kilobytes.
pub fn estimate_tree_size(chunks: &[Chunk]) -> u64 {
    let mut size: u64 = 0;

    chunks.iter().for_each(|chunk| size += chunk.size);

    size
}

fn get_chunk_filename(hash: &str, permissions: u32) -> String {
    let mut new_hash = hash.to_string();

    new_hash.push_str(&permissions.to_string());

    new_hash
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::MetadataExt;

    use super::*;

    use temp_dir::TempDir;

    #[test]
    fn get_chunk_filename_stability() {
        let hash = "a8sf799a8s6fa7f5";
        let permissions = 0o777;

        assert_eq!(get_chunk_filename(hash, permissions), "a8sf799a8s6fa7f5511");
    }

    #[test]
    fn test_save_tree() -> Result<()> {
        let initial_tree_path = TempDir::new()?;
        let chunk_store_path = TempDir::new()?;
        let hash_kind = &HashKind::Blake3;

        // Create example tree
        fs::write(initial_tree_path.path().join("file"), "Example")?;
        fs::create_dir(initial_tree_path.path().join("path"))?;
        fs::write(initial_tree_path.path().join("path/file"), "Example2")?;

        let chunks = save_tree(initial_tree_path.path(), chunk_store_path.path(), hash_kind)?;

        // Check that the correct number of chunks were created
        assert_eq!(chunks.len(), 2);

        // Check that the chunk hashes exist in the chunk store
        for chunk in &chunks {
            let chunk_path = chunk_store_path
                .path()
                .join(get_chunk_filename(&chunk.hash, chunk.permissions));
            assert!(
                chunk_path.exists(),
                "Chunk file does not exist: {:?}",
                chunk_path
            );
        }

        // Check that the chunk paths are correct
        let chunk_paths: Vec<_> = chunks
            .iter()
            .map(|c| c.path.to_string_lossy().to_string())
            .collect();
        assert!(chunk_paths.contains(&"file".to_string()));
        assert!(chunk_paths.contains(&"path/file".to_string()));

        // Check that the estimated size is correct (in KB)
        let expected_size = (b"Example".len() as u64) / 1024 + (b"Example2".len() as u64) / 1024;
        assert_eq!(estimate_tree_size(&chunks), expected_size);

        Ok(())
    }

    #[test]
    fn test_load_tree() -> Result<()> {
        let initial_tree_path = TempDir::new()?;
        let loaded_tree_path = TempDir::new()?;
        let chunk_store_path = TempDir::new()?;
        let hash_kind = &HashKind::Blake3;

        // Create example tree
        fs::write(initial_tree_path.path().join("file"), "Example")?;
        fs::create_dir(initial_tree_path.path().join("path"))?;
        fs::write(initial_tree_path.path().join("path/file"), "Example2")?;

        let chunks = save_tree(initial_tree_path.path(), chunk_store_path.path(), hash_kind)?;

        load_tree(loaded_tree_path.path(), chunk_store_path.path(), &chunks)?;

        assert_eq!(
            fs::read_to_string(loaded_tree_path.path().join("file"))?,
            "Example"
        );

        assert_eq!(
            fs::read_to_string(loaded_tree_path.path().join("path/file"))?,
            "Example2"
        );

        Ok(())
    }

    #[test]
    fn test_permissions() -> Result<()> {
        let initial_tree_path = TempDir::new()?;
        let loaded_tree_path = TempDir::new()?;
        let chunk_store_path = TempDir::new()?;
        let hash_kind = &HashKind::Blake3;

        // Create example tree
        let file_path = initial_tree_path.path().join("file");
        fs::write(&file_path, "Example")?;
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&file_path, perms)?;

        let file_path = initial_tree_path.path().join("file2");
        fs::write(&file_path, "Example")?;
        let mut perms2 = fs::metadata(&file_path)?.permissions();
        perms2.set_mode(0o600);
        fs::set_permissions(&file_path, perms2)?;

        let chunks = save_tree(initial_tree_path.path(), chunk_store_path.path(), hash_kind)?;

        load_tree(loaded_tree_path.path(), chunk_store_path.path(), &chunks)?;

        assert_eq!(
            fs::metadata(loaded_tree_path.path().join("file"))?.mode() & 0o777,
            0o700
        );

        assert_eq!(
            fs::metadata(loaded_tree_path.path().join("file2"))?.mode() & 0o777,
            0o600
        );

        Ok(())
    }

    #[test]
    fn test_tree_size() -> Result<()> {
        let initial_tree_path = TempDir::new()?;
        let chunk_store_path = TempDir::new()?;
        let hash_kind = &HashKind::Blake3;

        let kb1 = vec![0; 1024];
        let kb4 = vec![0; 4096];

        // Create example tree
        fs::write(initial_tree_path.path().join("file"), kb1)?;
        fs::create_dir(initial_tree_path.path().join("path"))?;
        fs::write(initial_tree_path.path().join("path/file"), kb4)?;

        let chunks = save_tree(initial_tree_path.path(), chunk_store_path.path(), hash_kind)?;

        // Check that the estimated size is correct (in KB)
        assert_eq!(estimate_tree_size(&chunks), 5);

        Ok(())
    }
}
