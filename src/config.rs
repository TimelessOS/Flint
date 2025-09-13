use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

pub fn get_config_dir() -> Result<PathBuf> {
    // Locate XDG config directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let config_dir: PathBuf = base_dirs.config_dir().join("flint");

    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn get_repos_dir() -> Result<PathBuf> {
    // Locate XDG data directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let config_dir: PathBuf = base_dirs.data_dir().join("flint");

    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn get_quicklaunch_dir() -> Result<PathBuf> {
    // Locate XDG data directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let config_dir: PathBuf = base_dirs.data_dir().join("flint-quicklaunch");

    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn system_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/var/lib/flint")
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/flint")
    }

    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\ProgramData\Flint")
    }
}

pub fn system_quicklaunch_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/var/lib/flint-quicklaunch")
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/flint-quicklaunch")
    }

    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\ProgramData\Flint-quicklaunch")
    }
}
