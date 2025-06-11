use anyhow::Result;
use lcas::{Store, create_store};
use std::path::PathBuf;

fn get_store() -> Store {
    let store = Store {
        kind: lcas::RepoType::Https,
        cache_path: get_cache_path(),
        path: get_store_path(),
        repo_path: get_repo_path(),
    };

    let _ = create_store(&store);

    store
}

fn get_store_path() -> PathBuf {
    PathBuf::new()
}

fn get_cache_path() -> PathBuf {
    PathBuf::new()
}

fn get_repo_path() -> String {
    "".to_string()
}

pub fn install(artifact_name: &String, repo: Option<String>) -> Result<()> {
    println!("{}", get_store().repo_path);
    Ok(())
}

pub fn repair() -> Result<()> {
    Ok(())
}

pub fn uninstall(artifact_name: &String) -> Result<()> {
    Ok(())
}

pub fn upgrade(artifact_name: &String) -> Result<()> {
    Ok(())
}

pub fn update_self() -> Result<()> {
    Ok(())
}
