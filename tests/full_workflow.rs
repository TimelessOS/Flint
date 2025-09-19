use anyhow::Result;
use std::path::Path;
use temp_dir::TempDir;

use flintpkg::{
    build::build,
    repo::{self, get_installed_package},
    run::{install, start},
};

#[tokio::test]
async fn full_workflow_test() -> Result<()> {
    let repo_dir = TempDir::new()?;
    let repo_path = repo_dir.path();
    let chunks_dir = TempDir::new()?;
    let chunks_path = chunks_dir.path();

    repo::create(repo_path, None)?;

    let build_manifest_path = Path::new("build_manifest.yml");
    build(build_manifest_path, repo_path, None, chunks_path).await?;

    install(repo_path, "example", chunks_path).await?;

    let manifest = get_installed_package(repo_path, "example")?;

    let args: Vec<&str> = vec!["--help"];
    let result = start(repo_path, manifest.clone(), "flint", args)?;
    assert!(result.success());

    let args: Vec<&str> = vec![];
    let result = start(repo_path, manifest, "flint", args)?;
    assert!(!result.success());

    Ok(())
}
