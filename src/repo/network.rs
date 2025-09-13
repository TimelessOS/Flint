use anyhow::Result;
use ed25519_dalek::VerifyingKey;
use std::path::Path;

use crate::{
    crypto::{key::deserialize_verifying_key, signing::verify_signature},
    log::{added_repo, cannot_update_repo, update_redirect},
    repo::{RepoManifest, manifest_io::atomic_replace, read_manifest, update_manifest},
};

/// Updates the Repository and returns a list of packages that have changed
///
/// # Errors
///
/// - Network Unavailable
/// - Server Unavailable
/// - Invalid signed data
pub async fn update_repository(repo_path: &Path) -> Result<bool> {
    let old_manifest = read_manifest(repo_path)?;

    if let Some(mirror) = old_manifest.mirrors.first() {
        let res_manifest = reqwest::get(format!("{mirror}/manifest.yml")).await?;
        let res_manifest_sig = reqwest::get(format!("{mirror}/manifest.yml.sig")).await?;

        let manifest = res_manifest.text().await?;
        let signature = res_manifest_sig.bytes().await?;

        let new_manifest = update_manifest(repo_path, &manifest, &signature)?;

        Ok(old_manifest != new_manifest)
    } else {
        Ok(false)
    }
}

/// Creates a Repository from a Remote Repository.
/// WILL REQUIRE USER INTERVENTION WITHOUT A PUBLIC KEY.
///
/// # Errors
///
/// - Network Unavailable
/// - Server Unavailable
/// - Invalid signed data
pub async fn add_repository(
    repo_path: &Path,
    mirror: &str,
    verifying_key: Option<VerifyingKey>,
) -> Result<RepoManifest> {
    let res_manifest = reqwest::get(format!("{mirror}/manifest.yml")).await?;
    let res_manifest_sig = reqwest::get(format!("{mirror}/manifest.yml.sig")).await?;

    let raw_manifest = res_manifest.text().await?;
    let signature = res_manifest_sig.bytes().await?;

    if let Some(verifying_key) = verifying_key {
        verify_signature(&raw_manifest, &signature, verifying_key)?;
    }

    // Make sure it actually deserializes
    let manifest: RepoManifest = serde_yaml::from_str(&raw_manifest)?;
    let repo_name = repo_path.file_name().unwrap_or_default();

    added_repo(repo_name, &manifest.public_key);

    if let Some(first_mirror) = manifest.mirrors.first() {
        if mirror != first_mirror {
            update_redirect(repo_name, first_mirror, mirror);
        }
    } else {
        cannot_update_repo(repo_name);
    }

    // VERIFY IT MATCHES ITSELF. IMPORTANT.
    verify_signature(
        &raw_manifest,
        &signature,
        deserialize_verifying_key(&manifest.public_key)?,
    )?;

    // Write to a .new, and then rename atomically
    atomic_replace(repo_path, "manifest.yml", raw_manifest.as_bytes())?;
    atomic_replace(repo_path, "manifest.yml.sig", &signature)?;

    Ok(manifest)
}
