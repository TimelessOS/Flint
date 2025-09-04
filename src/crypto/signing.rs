use std::{fs, path::Path};

use anyhow::Result;
use ed25519_dalek::{Signature, VerifyingKey, ed25519::signature::SignerMut};

use crate::crypto::key::get_private_key;

/// Signs and inserts the signature into the filesystem.
pub fn sign(repo_path: &Path, manifest_serialized: &str) -> Result<Signature> {
    let mut signing_key = get_private_key(None)?;
    let signature = signing_key.sign(manifest_serialized.as_bytes());

    fs::write(repo_path.join("manifest.yml.sig"), signature.to_bytes())?;

    verify_signature(
        manifest_serialized,
        &signature.to_bytes(),
        signing_key.verifying_key(),
    )?;

    Ok(signature)
}

/// Verifies the signature, and errors out if its incorrect
pub fn verify_signature(
    manifest_serialized: &str,
    signature: &[u8],
    verifying_key: VerifyingKey,
) -> Result<()> {
    let signature = Signature::try_from(signature)?;

    verifying_key.verify_strict(manifest_serialized.as_bytes(), &signature)?;

    Ok(())
}
