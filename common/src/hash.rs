
use sha2::{Sha256, Digest};

pub fn hash_bytes(data:&[u8]) -> String{
    let mut hasher = Sha256::new();
    hasher.update(data);
    let bytes = hasher.finalize().to_vec();
    hex::encode(&bytes)
}

pub fn hash_str(data:&str) -> String{
    hash_bytes(data.as_bytes())
}