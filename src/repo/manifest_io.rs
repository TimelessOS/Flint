use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    crypto::{key::deserialize_verifying_key, signing::verify_signature},
    repo::RepoManifest,
};

/// Reads a manifest and verifys it from the EXISTING key. This is best for GENERAL reading.
///
/// # Warning
/// Do NOT run on downloaded manifests before `read_manifest_signed`, or else potentially malicious inputs will be parsed.
///
/// # Errors
///
/// - Filesystem errors (Permissions or doesn't exist)
/// - Invalid signature
pub fn read_manifest(repo_path: &Path) -> Result<RepoManifest> {
    let manifest_serialized = fs::read_to_string(repo_path.join("manifest.yml"))?;
    let manifest_signature_serialized = fs::read(repo_path.join("manifest.yml.sig"))?;

    let manifest: RepoManifest = serde_yaml::from_str(&manifest_serialized)?;

    verify_signature(
        &manifest_serialized,
        &manifest_signature_serialized,
        deserialize_verifying_key(&manifest.public_key)?,
    )?;
    Ok(manifest)
}

fn read_manifest_unsigned(repo_path: &Path) -> Result<RepoManifest> {
    let manifest_serialized = fs::read_to_string(repo_path.join("manifest.yml"))?;

    let manifest: RepoManifest = serde_yaml::from_str(&manifest_serialized)?;

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

/// Replaces the existing manifest with another one, and verifies that it is correct
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
    atomic_replace(
        repo_path,
        "manifest.yml",
        new_manifest_serialized.as_bytes(),
    )?;
    atomic_replace(repo_path, "manifest.yml.sig", signature)?;

    Ok(())
}

fn atomic_replace(base_path: &Path, filename: &str, contents: &[u8]) -> Result<()> {
    let new_path = &base_path.join(filename.to_owned() + ".new");

    fs::write(new_path, contents)?;
    fs::rename(new_path, base_path.join(filename))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use temp_dir::TempDir;

    use crate::{crypto::signing::sign, repo::create};

    use super::*;

    #[test]
    fn test_atomic_replace_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        fs::write(temp_dir.path().join("file"), "previous_contents")?;
        atomic_replace(temp_dir.path(), "file", b"new_contents")?;

        assert_eq!(
            fs::read_to_string(temp_dir.path().join("file"))?,
            "new_contents"
        );
        assert!(!temp_dir.path().join("file.new").exists());

        Ok(())
    }

    #[test]
    fn test_update_manifest_valid_and_invalid() -> Result<()> {
        let repo = TempDir::new()?;
        let repo_path = repo.path();
        create(repo_path)?;

        let old_manifest = read_manifest(repo_path)?;

        // Build a new manifest with small change
        let mut new_manifest = old_manifest;
        new_manifest.metadata.title = Some("NewName".into());

        let serialized = serde_yaml::to_string(&new_manifest)?;

        // Sign it with the right key
        let signature = sign(repo_path, &serialized)?;

        // Update should succeed
        update_manifest(repo_path, &serialized, &signature.to_bytes())?;

        let updated = read_manifest(repo_path)?;
        assert_eq!(updated.metadata.title, Some("NewName".into()));

        // Now try with invalid signature
        let bad_signature = b"garbage_signature";
        assert!(update_manifest(repo_path, &serialized, bad_signature).is_err());

        Ok(())
    }

    #[test]
    fn test_read_signed_manifest() -> Result<()> {
        let repo = TempDir::new()?;
        let repo_path = repo.path();
        create(repo_path)?;

        let manifest = read_manifest(repo_path)?;
        let manifest_signed = read_manifest_signed(repo_path, &manifest.public_key)?;

        assert_eq!(manifest.edition, manifest_signed.edition);

        Ok(())
    }
}
