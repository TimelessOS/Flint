use anyhow::{Context, Result, anyhow};
use std::{env::current_exe, fs, os::unix::fs::PermissionsExt, path::Path};

use crate::repo::read_manifest;

/// Removes all nonexistant Quicklaunch items, and adds any missing ones.
///
/// # Errors
///
/// - Filesystem
/// - Bad Repositories
pub fn update_quicklaunch(repos_path: &Path, quicklaunch_path: &Path) -> Result<()> {
    let mut allowed = Vec::new();

    for entry in repos_path.read_dir()? {
        let repo_path = entry?.path();

        let manifest = read_manifest(&repo_path)?;

        for package in manifest.packages {
            for entrypoint in package.commands {
                let command = entrypoint
                    .file_name()
                    .ok_or_else(|| anyhow!("Could not get entrypoint name"))?;

                allowed.push(command.to_owned());

                let tmp_path = &quicklaunch_path.join(format!("{}.new", command.display()));
                let path = quicklaunch_path.join(command);

                // generate quicklaunch script
                let executable_path = current_exe()
                    .with_context(|| "Could not get current executable path")?
                    .canonicalize()?;
                let quicklaunch_script = format!(
                    "#!/bin/bash\n{} run {} {} $@",
                    executable_path.display(),
                    package.id,
                    command.display()
                );

                fs::write(tmp_path, quicklaunch_script)?;
                fs::rename(tmp_path, &path)?;

                // chmod 755
                let mut perms = fs::metadata(&path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&path, perms)?;
            }
        }
    }

    // delete quicklaunch scripts for removed things
    for entry in quicklaunch_path.read_dir()? {
        let file = entry?;

        if !allowed.contains(&file.file_name()) {
            fs::remove_file(file.path())?;
        }
    }

    Ok(())
}
