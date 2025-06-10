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

pub fn install(artifact_name: &String) {
    println!("{}", get_store().repo_path);
}

pub fn repair(artifact_name: &String) {}

pub fn uninstall(artifact_name: &String) {}

pub fn upgrade(artifact_name: &String) {}

pub fn update_self(artifact_name: &String) {}
