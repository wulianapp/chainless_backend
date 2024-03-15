use base58::{FromBase58, ToBase58};
use hex::FromHex;

pub fn bs58_to_hex(bs58_private_key: &str) -> Result<String, base58::FromBase58Error> {
    let bytes = bs58_private_key.from_base58()?;
    let hex_string = hex::encode(bytes);
    Ok(hex_string)
}

pub fn hex_to_bs58(hex_private_key: &str) -> Result<String, hex::FromHexError> {
    let bytes = Vec::from_hex(hex_private_key)?;
    let bs58_string = bytes.to_base58();
    Ok(bs58_string)
}

pub fn gen_random_verify_code() -> u32 {
    rand::random::<u32>() % 900000 + 100000
}
