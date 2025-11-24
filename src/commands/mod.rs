pub mod bundle;
pub mod main;
pub mod repo;

use anyhow::Result;
use flintpkg::{chunks::utils::clean_used, run::quicklaunch::update_quicklaunch};
use std::path::Path;

#[cfg(feature = "network")]
use crate::commands::main::update_cmd;
use crate::{
    Command,
    commands::{
        bundle::bundle_commands,
        main::{build_cmd, install_cmd, remove_cmd, run_cmd, verify_cmd},
        repo::repo_commands,
    },
};

pub async fn main_commands(
    base_path: &Path,
    quicklaunch_path: &Path,
    chunk_store_path: &Path,
    command: Command,
) -> Result<()> {
    match command {
        Command::Repo { command } => {
            repo_commands(base_path, chunk_store_path, command, quicklaunch_path).await?;
            update_quicklaunch(base_path, quicklaunch_path)?;
        }

        Command::Build {
            build_manifest_path,
            repo_name,
            force,
        } => {
            build_cmd(
                base_path,
                &repo_name,
                &build_manifest_path,
                chunk_store_path,
                force,
            )
            .await?;
        }

        Command::Install { repo_name, package } => {
            install_cmd(base_path, repo_name, chunk_store_path, &package).await?;
        }

        Command::Remove { repo_name, package } => remove_cmd(base_path, repo_name, &package)?,

        Command::Bundle { command } => bundle_commands(base_path, command)?,

        #[cfg(feature = "network")]
        Command::Update => update_cmd(base_path, quicklaunch_path, chunk_store_path).await?,

        Command::Run {
            repo_name,
            package,
            entrypoint,
            args,
        } => {
            run_cmd(
                base_path,
                repo_name,
                chunk_store_path,
                package,
                entrypoint,
                args,
            )
            .await?;
        }

        Command::VerifyChunks { repo_name } => verify_cmd(base_path, &repo_name, chunk_store_path)?,

        Command::Clean => clean_used(base_path, chunk_store_path)?,
    }

    Ok(())
}
