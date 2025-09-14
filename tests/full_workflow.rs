use std::path::Path;

use anyhow::Result;
use flintpkg::{
    build::build,
    repo,
    run::{install, start},
};
use temp_dir::TempDir;

#[tokio::test]
async fn full_workflow_test() -> Result<()> {
    let repo_dir = TempDir::new()?;
    let repo_path = repo_dir.path();

    repo::create(repo_path)?;

    let build_manifest_path = Path::new("build_manifest.yml");
    build(build_manifest_path, repo_path).await?;

    install(repo_path, "example").await?;

    let args: Vec<&str> = vec!["--help"];
    let result = start(repo_path, "example", "flint", args)?;
    assert!(result.success());

    let args: Vec<&str> = vec![];
    let result = start(repo_path, "example", "flint", args)?;
    assert!(!result.success());

    Ok(())
}
