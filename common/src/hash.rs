use core::fmt;

use sha2::{Digest, Sha256};

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize().to_vec();
    hex::encode(&bytes)
}

pub fn hash_str(data: &str) -> String {
    hash_bytes(data.as_bytes())
}

pub trait Hash: fmt::Display {
    fn hash(&self) -> String {
        hash_bytes(self.to_string().as_bytes())
    }
}

impl Hash for String {}
