/// WARNING: Only Blake3 is currently implemented for the time being.
pub enum HashKind {
    Blake3,
    Sha512,
    Sha256,
}

pub fn hash(hash_kind: &HashKind, data: &[u8]) -> String {
    match hash_kind {
        HashKind::Blake3 => blake3::hash(data).to_hex().to_string(),
        HashKind::Sha512 => "".to_string(),
        HashKind::Sha256 => "".to_string(),
    }
}
