#![warn(clippy::pedantic)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub arch: Option<String>,
}
