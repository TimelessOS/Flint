use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    crypto::{key::deserialize_verifying_key, signing::verify_signature},
    repo::RepoManifest,
};

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
    atomic_replace(
        repo_path,
        "manifest.yml",
        new_manifest_serialized.as_bytes(),
    )?;
    atomic_replace(repo_path, "manifest.yml.sig", signature)?;

    Ok(())
}

fn atomic_replace(repo_path: &Path, filename: &str, contents: &[u8]) -> Result<()> {
    let new_path = &repo_path.join(filename.to_owned() + ".new");

    fs::write(new_path, contents)?;
    fs::rename(new_path, repo_path.join(filename))?;

    Ok(())
}
