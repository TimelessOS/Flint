use console::style;
use flintpkg::repo::PackageManifest;
use std::{env::var_os, ffi::OsStr, path::Path};

pub fn skipped_update(package_name: &str) {
    println!(
        "[{}] Skipped Updating {}",
        style("SKIPPED").bright().black(),
        style(package_name).bright().green()
    );
}

pub fn skipped_update_repo(repo_name: &OsStr) {
    println!(
        "[{}] Skipped Updating Repository {}",
        style("SKIPPED").bright().black(),
        style(repo_name.display()).bright().green()
    );
}

pub fn updated_package(package: &PackageManifest) {
    let version_str = package
        .metadata
        .version
        .as_ref()
        .map_or_else(String::new, |version| {
            format!(" to {}", style(version).bright().yellow())
        });

    println!(
        "[{}] Updated {}{}",
        style("UPDATED").bright().green(),
        style(&package.id).bright().green(),
        version_str
    );
}

pub fn updated_repo(repo: &OsStr) {
    println!(
        "[{}] Updated Repository {}",
        style("UPDATED").bright().green(),
        style(&repo.display()).bright().green(),
    );
}

pub fn added_repo(repo: &str, public_key: &str) {
    println!(
        "[{}] Added Repository {} with public key: {public_key}",
        style("NOTICE").bright().green(),
        style(repo).bright().green(),
    );
}

pub fn cannot_update_repo(repo: &str) {
    println!(
        "[{}] This Repository has no mirrors: {}",
        style("CAUTION").bright().yellow(),
        style(&repo).bright().green(),
    );
}

pub fn update_redirect(repo: &str, old_url: &str, new_url: &str) {
    println!(
        "[{}] Updates will go to {} instead of {} for {}",
        style("CAUTION").bright().yellow(),
        style(old_url).bright().green(),
        style(new_url).bright().green(),
        style(&repo).bright().green(),
    );
}

pub fn add_to_path_notice(path: &Path) {
    let shell = var_os("SHELL")
        .and_then(|s| s.into_string().ok())
        .unwrap_or_default();

    let command = if shell.contains("fish") {
        format!("set -U fish_user_paths {} $fish_user_paths", path.display())
    } else if shell.contains("bash") || shell.contains("zsh") {
        let shell_cfg = if shell.contains("zsh") {
            "~/.zshrc"
        } else {
            "~/.bash_profile"
        };

        format!(
            "echo \"export PATH=\\$PATH:{}\" >> {} && source {}",
            path.display(),
            shell_cfg,
            shell_cfg
        )
    } else {
        format!(
            "# Add this line to your shell config:\nexport PATH=$PATH:{}",
            path.display()
        )
    };

    println!(
        "[{}] Please run the following command, or things will break:\n{}",
        console::style("WARNING").bright().yellow().bold(),
        console::style(command),
    );
}
