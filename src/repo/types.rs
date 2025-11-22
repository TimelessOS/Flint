use std::{collections::BTreeMap, path::PathBuf};

use crate::chunks::{Chunk, HashKind};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RepoManifest {
    pub metadata: Metadata,
    pub packages: Vec<PackageManifest>,
    pub public_key: String,
    pub mirrors: Vec<String>,
    pub edition: String,
    pub hash_kind: HashKind,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub metadata: Metadata,
    pub id: String,
    pub aliases: Vec<String>,
    pub chunks: Vec<Chunk>,
    pub commands: Vec<PathBuf>,
    /// Runtime environment variables
    pub env: Option<BTreeMap<String, String>>,
}

/// All of these are user visible, and should carry no actual weight.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub homepage_url: Option<String>,
    /// User visible, not actually used to compare versions
    pub version: Option<String>,
    /// SPDX Identifier
    pub license: Option<String>,
}
