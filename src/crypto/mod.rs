use anyhow::Result;
use ed25519_dalek::{VerifyingKey, pkcs8::DecodePublicKey};

pub mod key;
pub mod signing;

pub fn get_public_key_from_pem(pem: &str) -> Result<VerifyingKey> {
    let verifying_key = VerifyingKey::from_public_key_pem(pem)
        .map_err(|e| anyhow::anyhow!("failed to encode public key: {e}"))?;

    Ok(verifying_key)
}
