use anyhow::{Result, anyhow};
use comfy_table::Table;
use flintpkg::chunks::utils::clean_unused;
use std::{fs, os::unix::fs::symlink, path::Path};

use crate::RepoCommands;
use flintpkg::{
    crypto::signing::sign,
    repo::{self, read_manifest, remove_package, update_manifest},
    utils::resolve_repo,
};

pub async fn repo_commands(
    base_path: &Path,
    chunk_store_path: &Path,
    command: RepoCommands,
) -> Result<()> {
    match command {
        RepoCommands::Create { repo_name } => {
            let repo_path = &base_path.join(&repo_name);

            repo::create(repo_path, None)?;
            symlink(Path::new("../../chunks"), repo_path.join("chunks"))?;
        }

        RepoCommands::List => {
            let mut table = Table::new();

            table.set_header(vec![
                "Name",
                "Title",
                "Hash Kind",
                "Homepage",
                "License",
                "Version",
            ]);

            for repo_entry in fs::read_dir(base_path)? {
                let repo_dir = repo_entry?;
                let repo_name = repo_dir.file_name();
                let repo_name_str = repo_name
                    .to_str()
                    .ok_or_else(|| anyhow!("Repository {} is not unicode.", repo_name.display()))?;

                let repo = read_manifest(&repo_dir.path())?;

                table.add_row(vec![
                    &repo_name_str,
                    repo.metadata.title.unwrap_or_default().as_str(),
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
            use crate::log::{added_repo, cannot_update_repo, update_redirect};
            use flintpkg::repo::network::add_repository;

            let repo_path = &base_path.join(&repo_name);
            fs::create_dir_all(repo_path)?;

            let manifest = add_repository(repo_path, &remote_url, None).await?;
            added_repo(&repo_name, &manifest.public_key);

            if let Some(first_mirror) = manifest.mirrors.first() {
                if remote_url != *first_mirror {
                    update_redirect(&repo_name, first_mirror, &remote_url);
                }
            } else {
                cannot_update_repo(&repo_name);
            }
        }

        RepoCommands::Remove { repo_name } => {
            fs::remove_dir_all(resolve_repo(base_path, &repo_name)?)?;
        }

        RepoCommands::Update {
            homepage_url,
            license,
            title,
            version,
            repo_name,
            mirrors,
        } => {
            let repo_path = &resolve_repo(base_path, &repo_name)?;
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
            remove_package(&package_id, &resolve_repo(base_path, &repo_name)?, None)?;
            clean_unused(base_path, chunk_store_path)?;
        }
    }

    Ok(())
}
