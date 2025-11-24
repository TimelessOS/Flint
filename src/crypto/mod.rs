use ed25519_dalek::{SecretKey, SigningKey};

pub mod key;
pub mod signing;

fn generate_signing_key() -> SigningKey {
    let mut secret = SecretKey::default();
    getrandom::fill(&mut secret)
        .expect("could not get random bytes from system RNG. Kernel error?");
    SigningKey::from_bytes(&secret)
}
