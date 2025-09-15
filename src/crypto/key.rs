use anyhow::Result;
use ed25519_dalek::{
    SigningKey, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
        spki::der::pem::LineEnding,
    },
};
use rand_core::{OsRng, UnwrapErr};
use std::{
    fs::{self, create_dir_all},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::config::get_config_dir;

/// Returns private key, generating it if necessary
pub fn get_private_key(config_path: Option<&Path>) -> Result<SigningKey> {
    let path = unwrap_config_path(config_path)?.join("id_ed25519");

    if !unwrap_config_path(config_path)?.exists() {
        create_dir_all(unwrap_config_path(config_path)?)?;
    }

    if !path.exists() {
        let mut csprng = UnwrapErr(OsRng);
        let signing_key = SigningKey::generate(&mut csprng);

        let pem = signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("failed to encode private key: {e}"))?;
        fs::write(&path, pem)?;

        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    let pem_str = fs::read_to_string(&path)?;
    let key = SigningKey::from_pkcs8_pem(&pem_str)
        .map_err(|e| anyhow::anyhow!("failed to decode private key: {e}"))?;

    Ok(key)
}

fn unwrap_config_path(config_path: Option<&Path>) -> Result<PathBuf> {
    let path = if let Some(config_path) = config_path {
        config_path.to_path_buf()
    } else {
        get_config_dir()?
    };

    Ok(path)
}

pub fn serialize_verifying_key(verifying_key: VerifyingKey) -> Result<String> {
    let pem = verifying_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| anyhow::anyhow!("failed to encode public key: {e}"))?;

    Ok(pem)
}

pub fn deserialize_verifying_key(verifying_key_serialized: &str) -> Result<VerifyingKey> {
    let verifying_key = VerifyingKey::from_public_key_pem(verifying_key_serialized)
        .map_err(|e| anyhow::anyhow!("failed to decode public key: {e}"))?;

    Ok(verifying_key)
}

#[cfg(test)]
mod tests {
    use temp_dir::TempDir;

    use super::*;

    #[test]
    fn test_generate_private_key_and_read_back() -> Result<()> {
        let temp = TempDir::new().unwrap();
        let config_dir = &temp.path().join("flint");

        // Create key
        let path = config_dir.join("id_ed25519");

        // Should generate new private key
        let key = get_private_key(Some(config_dir))?;
        assert!(path.exists());
        assert!(!key.verifying_key().to_bytes().is_empty());

        // Should load the same key on second call
        let key2 = get_private_key(Some(config_dir))?;
        assert_eq!(key.to_bytes(), key2.to_bytes());

        Ok(())
    }

    #[test]
    fn test_generate_private_key_permissions() -> Result<()> {
        let temp = TempDir::new().unwrap();
        let config_dir = &temp.path().join("flint");

        // Create key
        let path = config_dir;

        // Generate new private key
        let _ = get_private_key(Some(path))?;

        let permissions = fs::metadata(path.join("id_ed25519"))?.permissions().mode();
        assert_eq!(permissions & 0o777, 0o600);

        Ok(())
    }
}
