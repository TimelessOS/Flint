use std::{fs, path::Path};

use anyhow::Result;

use crate::chunks::{Chunk, HashKind};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct RepoManifest {
    pub metadata: Metadata,
    pub packages: Vec<PackageManifest>,
    pub updates_url: Option<String>,
    pub public_key: String,
    pub mirrors: Vec<String>,
    edition: String,
    pub hash_kind: HashKind,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct PackageManifest {
    pub metadata: Metadata,
    pub id: String,
    pub aliases: Vec<String>,
    pub chunks: Vec<Chunk>,
    pub commands: Vec<String>,
}

/// All of these are user visible, and should carry no actual weight.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Metadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    /// User visible, not actually used to compare versions
    pub version: Option<String>,
    /// SPDX Identifier
    pub license: Option<String>,
}

pub fn create(repo_path: &Path) -> Result<()> {
    let manifest = RepoManifest {
        edition: "2025".into(),
        hash_kind: HashKind::Blake3,
        metadata: Metadata {
            name: None,
            description: None,
            homepage_url: None,
            version: None,
            license: None,
        },
        mirrors: Vec::new(),
        updates_url: None,
        packages: Vec::new(),
        public_key: "".into(),
    };

    let manifest_serialized = serde_yaml::to_string(&manifest)?;
    fs::write(repo_path.join("manifest.yml"), manifest_serialized)?;

    Ok(())
}
