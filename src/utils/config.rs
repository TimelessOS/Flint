use anyhow::{Context, Result};
use directories::BaseDirs;
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::{
    SigningKey, pkcs8::DecodePrivateKey, pkcs8::DecodePublicKey, pkcs8::EncodePrivateKey,
    pkcs8::EncodePublicKey,
};
use rand_core::{OsRng, TryRngCore};
use std::fs;
use std::path::{Path, PathBuf};

fn get_config_dir() -> Result<PathBuf> {
    // Locate XDG config directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let config_dir: PathBuf = base_dirs.config_dir().join("flint");

    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

/// Returns private key, generating it if necessary
pub fn get_private_key(config_path: Option<&Path>) -> Result<SigningKey> {
    let path = if let Some(config_path) = config_path {
        config_path.join("id_ed25519")
    } else {
        get_config_dir()?.join("id_ed25519")
    };

    if !path.exists() {
        let new_key = generate_private_key();
        let pem = new_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("failed to encode private key: {e}"))?;
        fs::write(&path, pem)?;
    }

    let pem_str = fs::read_to_string(&path)?;
    let key = SigningKey::from_pkcs8_pem(&pem_str)
        .map_err(|e| anyhow::anyhow!("failed to decode private key: {e}"))?;

    Ok(key)
}

fn generate_private_key() -> SigningKey {
    let mut csprng = OsRng.unwrap_err();
    SigningKey::generate(&mut csprng)
}

/// Returns public key, generating it AND/OR the private key if necessary
pub fn get_public_key(config_path: Option<&Path>) -> Result<VerifyingKey> {
    let path = if let Some(config_path) = config_path {
        config_path.join("id_ed25519")
    } else {
        get_config_dir()?.join("id_ed25519")
    };

    if !path.exists() {
        let new_key = generate_public_key(&path)?;
        let pem = new_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("failed to encode public key: {e}"))?;
        fs::write(&path, pem)?;
    }

    let pem_str = fs::read_to_string(&path)?;
    let key = VerifyingKey::from_public_key_pem(&pem_str)
        .map_err(|e| anyhow::anyhow!("failed to decode public key: {e}"))?;

    Ok(key)
}

fn generate_public_key(config_path: &Path) -> Result<VerifyingKey> {
    let signing_key = get_private_key(Some(config_path))?;
    let public_key = signing_key.verifying_key();

    Ok(public_key)
}

#[cfg(test)]
mod tests {
    use temp_dir::TempDir;

    use super::*;

    /// Override config dir to a temporary one
    fn with_temp_config<F, T>(f: F) -> Result<T>
    where
        F: FnOnce(&PathBuf) -> Result<T>,
    {
        let temp = TempDir::new().unwrap();
        let config_dir = temp.path().join("flint");
        fs::create_dir_all(&config_dir)?;
        f(&config_dir)
    }

    #[test]
    fn test_generate_private_key_and_read_back() -> Result<()> {
        with_temp_config(|config_dir| {
            // Create key
            let path = config_dir.join("id_ed25519");

            // Should generate new private key
            let key = get_private_key_from_path(&path)?;
            assert!(path.exists());
            assert!(!key.verifying_key().to_bytes().is_empty());

            // Should load the same key on second call
            let key2 = get_private_key_from_path(&path)?;
            assert_eq!(key.to_bytes(), key2.to_bytes());

            Ok(())
        })
    }

    #[test]
    fn test_generate_public_key_and_read_back() -> Result<()> {
        with_temp_config(|config_dir| {
            let priv_path = config_dir.join("id_ed25519");
            let pub_path = config_dir.join("id_ed25519.pub");

            // Generate private+public
            let priv_key = get_private_key_from_path(&priv_path)?;
            let pub_key = get_public_key_from_path(&pub_path, &priv_path)?;

            assert!(pub_path.exists());
            assert_eq!(priv_key.verifying_key().to_bytes(), pub_key.to_bytes());

            Ok(())
        })
    }

    // --- Helper fns for tests that bypass global get_config_dir ---
    fn get_private_key_from_path(path: &PathBuf) -> Result<SigningKey> {
        if !path.exists() {
            let new_key = generate_private_key();
            let pem = new_key.to_pkcs8_pem(LineEnding::LF).unwrap();
            fs::write(path, pem)?;
        }
        let pem_str = fs::read_to_string(path)?;
        let key = SigningKey::from_pkcs8_pem(&pem_str).unwrap();
        Ok(key)
    }

    fn get_public_key_from_path(pub_path: &PathBuf, priv_path: &PathBuf) -> Result<VerifyingKey> {
        if !pub_path.exists() {
            let signing_key = get_private_key_from_path(priv_path)?;
            let new_key = signing_key.verifying_key();
            let pem = new_key.to_public_key_pem(LineEnding::LF).unwrap();
            fs::write(pub_path, pem)?;
        }
        let pem_str = fs::read_to_string(pub_path)?;
        let key = VerifyingKey::from_public_key_pem(&pem_str).unwrap();
        Ok(key)
    }
}
