use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

/// Gets the default/main configuration directory
///
/// # Errors
///
/// - No valid home directory path could be retrieved from the operating system.
/// - Config dir could not be created
pub fn get_config_dir() -> Result<PathBuf> {
    // Locate XDG config directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let config_dir: PathBuf = base_dirs.config_dir().join("flint");

    if !&config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

/// Gets the default/main USER repositorys directory
///
/// # Errors
///
/// - No valid home directory path could be retrieved from the operating system.
/// - Reposiorys dir could not be created
pub fn get_repos_dir() -> Result<PathBuf> {
    // Locate XDG data directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let repos_dir: PathBuf = base_dirs.data_dir().join("flint");

    if !&repos_dir.exists() {
        fs::create_dir_all(&repos_dir)?;
    }

    Ok(repos_dir)
}

/// Gets the main quicklaunch directory
///
/// # Errors
///
/// - No valid home directory path could be retrieved from the operating system.
/// - Quicklaunch dir could not be created
pub fn get_quicklaunch_dir() -> Result<PathBuf> {
    // Locate XDG data directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let quicklaunch_dir: PathBuf = base_dirs.data_dir().join("flint-quicklaunch");

    if !&quicklaunch_dir.exists() {
        fs::create_dir_all(&quicklaunch_dir)?;
    }

    Ok(quicklaunch_dir)
}

/// Gets the build cache directory
///
/// # Errors
///
/// - No valid home directory path could be retrieved from the operating system.
/// - Cache dir could not be created
pub fn get_build_cache_dir() -> Result<PathBuf> {
    // Locate XDG config directory
    let base_dirs = BaseDirs::new().context("Could not find user directories")?;
    let build_cache_dir: PathBuf = base_dirs.cache_dir().join("flint");

    if !&build_cache_dir.exists() {
        fs::create_dir_all(&build_cache_dir)?;
    }

    Ok(build_cache_dir)
}

#[must_use]
/// Gets the SYSTEM-WIDE Repositorys path
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

#[must_use]
/// Gets the SYSTEM-WIDE quicklaunch path
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
