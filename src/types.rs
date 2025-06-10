use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    metadata: Metadata,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Metadata {
    name: String,
    description: Option<String>,
    url: Option<String>,
    license: Option<String>,
    arch: Option<String>,
}
