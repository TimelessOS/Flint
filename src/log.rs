use crate::repo::PackageManifest;
use console::style;
use std::ffi::OsStr;

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

pub fn added_repo(repo: &OsStr, public_key: &str) {
    println!(
        "[{}] Added Repository {} with public key: {public_key}",
        style("NOTICE").bright().green(),
        style(&repo.display()).bright().green(),
    );
}

pub fn cannot_update_repo(repo: &OsStr) {
    println!(
        "[{}] This Repository has no mirrors: {}",
        style("CAUTION").bright().yellow(),
        style(&repo.display()).bright().green(),
    );
}

pub fn update_redirect(repo: &OsStr, old_url: &str, new_url: &str) {
    println!(
        "[{}] Updates will go to {} instead of {} for {}",
        style("CAUTION").bright().yellow(),
        style(old_url).bright().green(),
        style(new_url).bright().green(),
        style(&repo.display()).bright().green(),
    );
}
