use crate::build::Source;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub async fn get_sources(path: &Path, source_path: &Path, sources: &[Source]) -> Result<()> {
    for source in sources {
        match source.kind.as_str() {
            "git" => pull_git(source, path).with_context(|| {
                format!("Failed to pull git repo from {}", source_path.display())
            })?,

            #[cfg(feature = "network")]
            "tar" => pull_tar(source, path).await.with_context(|| {
                format!(
                    "Failed to extract tar archive from {}",
                    source_path.display()
                )
            })?,

            "local" => pull_local(source_path, path).with_context(|| {
                format!("Failed to copy local source from {}", source_path.display())
            })?,
            _ => {
                unimplemented!("No handler is implemented for source.kind.{}", source.kind)
            }
        }
    }

    Ok(())
}

/// Just copy files from a local path into the target.
/// If target already exists, nuke it first.
fn pull_local(source_path: &Path, target_path: &Path) -> Result<()> {
    // Remove target if it already exists
    if target_path.exists() {
        fs::remove_dir_all(target_path)
            .with_context(|| format!("Failed to remove existing dir {}", target_path.display()))?;
    }

    fs::create_dir_all(target_path)
        .with_context(|| format!("Failed to create target dir {}", target_path.display()))?;

    // Copy recursively
    for entry in walkdir::WalkDir::new(source_path) {
        let entry = entry?;
        let rel_path = entry.path().strip_prefix(source_path)?;
        let dest = target_path.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }

    Ok(())
}

/// Clone or pull a git repo depending on whether it already exists.
fn pull_git(source: &Source, target_path: &Path) -> Result<()> {
    // Clone fresh
    let status = Command::new("git")
        .arg("clone")
        .arg(&source.url)
        .arg(target_path)
        .status()
        .with_context(|| "Failed to run git clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }

    if let Some(commit) = &source.commit {
        let status = Command::new("git")
            .arg("checkout")
            .arg(commit)
            .current_dir(target_path)
            .status()
            .with_context(|| "Failed to run git clone")?;
        if !status.success() {
            anyhow::bail!("git checkout failed");
        }
    }

    Ok(())
}

#[cfg(feature = "network")]
async fn pull_tar(source: &Source, target_path: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    if target_path.exists() {
        fs::remove_dir_all(target_path)?;
    }

    fs::create_dir_all(target_path)?;

    let res = reqwest::get(&source.url)
        .await
        .with_context(|| format!("Failed to fetch tarball from {}", source.url))?
        .error_for_status()
        .with_context(|| format!("HTTP error fetching {}", source.url))?;

    let bytes = res
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;
    let cursor = Cursor::new(bytes);

    if let Some(extension) = Path::new(&source.url).extension() {
        // Detect gzip by extension
        if extension.eq_ignore_ascii_case("tar.gz") || extension.eq_ignore_ascii_case("tgz") {
            let tar = GzDecoder::new(cursor);
            let mut archive = Archive::new(tar);
            archive
                .unpack(target_path)
                .with_context(|| format!("Failed to unpack gzip tar from {}", source.url))?;
        } else {
            let mut archive = Archive::new(cursor);
            archive
                .unpack(target_path)
                .with_context(|| format!("Failed to unpack tar from {}", source.url))?;
        }
        Ok(())
    } else {
        use anyhow::bail;

        bail!("No extension on tar source url.")
    }
}
