use std::{fs, path::Path};

use anyhow::Result;
use ed25519_dalek::{Signature, VerifyingKey, ed25519::signature::Signer};

use crate::crypto::key::get_private_key;

/// Signs and inserts the signature into the filesystem.
pub fn sign(repo_path: &Path, manifest_serialized: &str) -> Result<Signature> {
    let signing_key = get_private_key(None)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand_core::{OsRng, TryRngCore};

    #[test]
    fn test_sign_and_verify() -> Result<()> {
        let manifest = "test manifest content";

        // Generate a key for test
        let mut csprng = OsRng.unwrap_err();
        let signing_key = SigningKey::generate(&mut csprng);

        // Sign manually
        let signature = signing_key.sign(manifest.as_bytes());

        // Verify
        verify_signature(manifest, &signature.to_bytes(), signing_key.verifying_key())?;

        Ok(())
    }

    #[test]
    fn test_verify_signature_invalid() {
        let manifest = "test";
        let invalid_sig = [0u8; 64];
        let mut csprng = OsRng.unwrap_err();
        let signing_key = SigningKey::generate(&mut csprng);

        let result = verify_signature(manifest, &invalid_sig, signing_key.verifying_key());

        assert!(result.is_err());
    }
}
