use anyhow::{Result, anyhow};
use comfy_table::Table;
use std::{fs, path::Path};

use crate::{
    RepoCommands,
    crypto::signing::sign,
    repo::{self, read_manifest, remove_package, update_manifest},
    utils::resolve_repo,
};

pub async fn repo_commands(path: &Path, command: RepoCommands) -> Result<()> {
    match command {
        RepoCommands::Create { repo_name } => repo::create(&path.join(&repo_name), None)?,

        RepoCommands::List => {
            let mut table = Table::new();

            table.set_header(vec![
                "Name",
                "Title",
                "URL",
                "Hash Kind",
                "Homepage",
                "License",
                "Version",
            ]);

            for repo_entry in fs::read_dir(path)? {
                let repo_dir = repo_entry?;
                let repo_name = repo_dir.file_name();
                let repo_name_str = repo_name
                    .to_str()
                    .ok_or_else(|| anyhow!("Repository {} is not unicode.", repo_name.display()))?;

                let repo = read_manifest(&repo_dir.path())?;

                table.add_row(vec![
                    &repo_name_str,
                    repo.metadata.title.unwrap_or_default().as_str(),
                    &repo.updates_url.unwrap_or_default(),
                    &repo.hash_kind.to_string(),
                    &repo.metadata.homepage_url.unwrap_or_default(),
                    &repo.metadata.license.unwrap_or_default(),
                    &repo.metadata.version.unwrap_or_default(),
                ]);
            }

            println!("{table}");
        }

        #[cfg(feature = "network")]
        RepoCommands::Add {
            repo_name,
            remote_url,
        } => {
            use crate::repo::network::add_repository;

            let repo_path = &path.join(repo_name);
            fs::create_dir_all(repo_path)?;

            add_repository(repo_path, &remote_url, None).await?;
        }

        RepoCommands::Remove { repo_name } => {
            fs::remove_dir_all(resolve_repo(path, &repo_name)?)?;
        }

        RepoCommands::Update {
            homepage_url,
            license,
            title,
            version,
            repo_name,
            mirrors,
        } => {
            let repo_path = &resolve_repo(path, &repo_name)?;
            let mut repo = read_manifest(repo_path)?;

            if title.is_some() {
                repo.metadata.title = title;
            }
            if homepage_url.is_some() {
                repo.metadata.homepage_url = homepage_url;
            }
            if license.is_some() {
                repo.metadata.license = license;
            }
            if version.is_some() {
                repo.metadata.version = version;
            }
            if let Some(mirrors) = mirrors {
                repo.mirrors = mirrors
                    .split(',')
                    .map(std::string::ToString::to_string)
                    .collect();
            }

            let manifest_serialized = &serde_yaml::to_string(&repo)?;
            let signature = sign(repo_path, manifest_serialized, None)?;

            update_manifest(repo_path, manifest_serialized, &signature.to_bytes())?;
        }

        RepoCommands::RemovePackage {
            repo_name,
            package_id,
        } => {
            remove_package(&package_id, &resolve_repo(path, &repo_name)?, None)?;
        }
    }

    Ok(())
}
