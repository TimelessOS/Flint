use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

use super::BuildManifest;
use crate::repo::{get_package, read_manifest};

/// Get the `build_hash` of a `build_manifest`
/// Requires all dependencies to be built and in the Repository beforehand.
///
/// # Errors
///
/// - Scripts do not exist
/// - Invalid build manifest
pub fn calc_build_hash(build_manifest_path: &Path, repo_path: &Path) -> Result<String> {
    let build_manifest_path = build_manifest_path.canonicalize().with_context(
        || "could not canoncicalize build manifest path. Does the build manifest exist?",
    )?;
    let build_manifest_raw = fs::read_to_string(build_manifest_path)?;
    let build_manifest: BuildManifest = serde_yaml::from_str(&build_manifest_raw)?;

    let repo_manifest = read_manifest(repo_path)?;

    let mut hash = blake3::Hasher::new();

    hash.write_all(build_manifest_raw.as_bytes())?;

    // Hash the `includes`
    if let Some(deps) = build_manifest.include {
        for dep in deps {
            let package = get_package(&repo_manifest, &dep)?;
            hash.write_all(package.build_hash.as_bytes())?;
        }
    }

    // Hash the `sdks`
    if let Some(deps) = build_manifest.sdks {
        for dep in deps {
            let package = get_package(&repo_manifest, &dep)?;
            hash.write_all(package.build_hash.as_bytes())?;
        }
    }

    // Hash the `build_script`
    if let Some(build_script) = build_manifest.build_script {
        let script = fs::read_to_string(build_script)?;
        hash.write_all(script.as_bytes())?;
    }

    // Hash the `post_script`
    if let Some(post_script) = build_manifest.post_script {
        let script = fs::read_to_string(post_script)?;
        hash.write_all(script.as_bytes())?;
    }

    Ok(hash.finalize().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use temp_dir::TempDir;

    use super::*;
    use crate::repo::{Metadata, create_repo};

    #[test]
    fn test_build_hash_stability() {
        let manifest = BuildManifest {
            id: "test_package".into(),
            aliases: Vec::new(),
            metadata: Metadata {
                description: None,
                homepage_url: None,
                title: None,
                version: None,
                license: None,
            },
            commands: Vec::new(),
            directory: PathBuf::from("."),
            edition: "2025".into(),
            build_script: None,
            post_script: None,
            sources: None,
            include: None,
            sdks: None,
            env: None,
        };

        let repo = TempDir::new().unwrap();
        create_repo(repo.path(), None).unwrap();

        let manifest_path = repo.path().join("build_manifest.yml");

        fs::write(&manifest_path, serde_yaml::to_string(&manifest).unwrap()).unwrap();

        let known_hash = "680cec2b6b847e76d733fb435214b18ec2108e25b4dfc54695f5daa1e987ec8d";
        let calc_hash = calc_build_hash(&manifest_path, repo.path()).unwrap();

        assert_eq!(known_hash, calc_hash);
    }
}
