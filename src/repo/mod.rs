use std::fs::create_dir_all;
use std::{fs, path::Path};

use anyhow::{Result, bail};

use crate::chunks::{Chunk, HashKind};
use crate::crypto::key::{deserialize_verifying_key, get_public_key, serialize_verifying_key};
use crate::crypto::signing::{sign, verify_signature};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct RepoManifest {
    pub metadata: Metadata,
    pub packages: Vec<PackageManifest>,
    pub updates_url: Option<String>,
    pub public_key: String,
    pub mirrors: Vec<String>,
    edition: String,
    pub hash_kind: HashKind,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct PackageManifest {
    pub metadata: Metadata,
    pub id: String,
    pub aliases: Vec<String>,
    pub chunks: Vec<Chunk>,
    pub commands: Vec<String>,
}

/// All of these are user visible, and should carry no actual weight.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    /// User visible, not actually used to compare versions
    pub version: Option<String>,
    /// SPDX Identifier
    pub license: Option<String>,
}

/// Creates a repository at `repo_path`
///
/// # Errors
///
/// - File permission errors at `repo_path`
/// - Key generation errors (If you do not already have a key)
pub fn create(repo_path: &Path) -> Result<()> {
    if repo_path.join("manifest.yml").exists() {
        bail!("Repository Already exists")
    }
    create_dir_all(repo_path)?;

    let manifest = RepoManifest {
        edition: "2025".into(),
        hash_kind: HashKind::Blake3,
        metadata: Metadata {
            name: None,
            description: None,
            homepage_url: None,
            version: None,
            license: None,
        },
        mirrors: Vec::new(),
        updates_url: None,
        packages: Vec::new(),
        public_key: serialize_verifying_key(get_public_key(None)?)?,
    };

    let manifest_serialized = serde_yaml::to_string(&manifest)?;
    fs::write(repo_path.join("manifest.yml"), &manifest_serialized)?;

    sign(repo_path, &manifest_serialized)?;

    Ok(())
}

/// Reads a manifest without verifying. This is best for AFTER it has been downloaded.
///
/// # Errors
///
/// - Filesystem errors (Permissions or doesn't exist)
pub fn read_manifest_unsigned(repo_path: &Path) -> Result<RepoManifest> {
    let manifest_serialized = fs::read_to_string(repo_path.join("manifest.yml"))?;
    let manifest = serde_yaml::from_str(&manifest_serialized)?;

    Ok(manifest)
}

/// Reads a manifest and verifys it. This is best for WHEN it has been downloaded.
///
/// # Errors
///
/// - Filesystem errors (Permissions or doesn't exist)
/// - Invalid signature
pub fn read_manifest_signed(repo_path: &Path, public_key_serialized: &str) -> Result<RepoManifest> {
    let manifest_serialized = fs::read_to_string(repo_path.join("manifest.yml"))?;
    let manifest_signature_serialized = fs::read(repo_path.join("manifest.yml.sig"))?;

    verify_signature(
        &manifest_serialized,
        &manifest_signature_serialized,
        deserialize_verifying_key(public_key_serialized)?,
    )?;

    let manifest = serde_yaml::from_str(&manifest_serialized)?;
    Ok(manifest)
}

/// Replaces the existing manifest with another one
/// Verifies that it is correct
///
/// # Errors
///
/// - Invalid Signature
/// - Filesystem error when updating (Out of space, Permissions)
/// - New manifest is invalid
pub fn update_manifest(
    repo_path: &Path,
    new_manifest_serialized: &str,
    signature: &[u8],
) -> Result<()> {
    let old_manifest = read_manifest_unsigned(repo_path)?;

    // VERIFY. IMPORTANT.
    verify_signature(
        new_manifest_serialized,
        signature,
        deserialize_verifying_key(&old_manifest.public_key)?,
    )?;

    // Make sure it actually deserializes
    let _: RepoManifest = serde_yaml::from_str(new_manifest_serialized)?;

    // Write to a .new, and then rename atomically
    fs::write(repo_path.join("manifest.yml.new"), new_manifest_serialized)?;
    fs::write(
        repo_path.join("manifest.yml.sig.new"),
        new_manifest_serialized,
    )?;

    fs::rename(
        repo_path.join("manifest.yml.new"),
        repo_path.join("manifest.yml"),
    )?;
    fs::rename(
        repo_path.join("manifest.yml.sig.new"),
        repo_path.join("manifest.yml.sig"),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use temp_dir::TempDir;

    use super::*;
    use std::fs;

    #[test]
    fn test_create_and_read_unsigned() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path();

        // Create repo
        create(repo_path).unwrap();

        // Read unsigned manifest
        let manifest = read_manifest_unsigned(repo_path).unwrap();
        assert_eq!(manifest.edition, "2025");
        assert!(manifest.public_key.len() > 10);
        assert!(manifest.packages.is_empty());

        // Should have manifest.yml + .sig
        assert!(repo_path.join("manifest.yml").exists());
        assert!(repo_path.join("manifest.yml.sig").exists());
    }

    #[test]
    fn test_read_signed_manifest() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path();
        create(repo_path).unwrap();

        let manifest = read_manifest_unsigned(repo_path).unwrap();
        let manifest_signed = read_manifest_signed(repo_path, &manifest.public_key).unwrap();

        assert_eq!(manifest.edition, manifest_signed.edition);
    }

    #[test]
    fn test_tampered_manifest_fails() -> Result<()> {
        let tmp = TempDir::new()?;
        let repo_path = tmp.path();
        create(repo_path)?;

        // Tamper with manifest.yml
        let mut contents = fs::read_to_string(repo_path.join("manifest.yml"))?;
        contents.push_str("\n# sneaky hacker change");
        fs::write(repo_path.join("manifest.yml"), contents)?;

        let manifest = read_manifest_unsigned(repo_path)?;
        let result = read_manifest_signed(repo_path, &manifest.public_key);

        assert!(
            result.is_err(),
            "tampered manifest should fail verification"
        );

        Ok(())
    }

    #[test]
    fn test_update_manifest_valid_and_invalid() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path();
        create(repo_path).unwrap();

        let old_manifest = read_manifest_unsigned(repo_path).unwrap();

        // Build a new manifest with small change
        let mut new_manifest = old_manifest;
        new_manifest.metadata.name = Some("NewName".into());

        let serialized = serde_yaml::to_string(&new_manifest).unwrap();

        // Sign it with the right key
        let signature = sign(repo_path, &serialized).unwrap();

        // Update should succeed
        update_manifest(repo_path, &serialized, &signature.to_bytes()).unwrap();

        let updated = read_manifest_unsigned(repo_path).unwrap();
        assert_eq!(updated.metadata.name, Some("NewName".into()));

        // Now try with invalid signature
        let bad_sig = b"garbage_signature";
        let result = update_manifest(repo_path, &serialized, bad_sig);
        assert!(result.is_err());
    }
}
