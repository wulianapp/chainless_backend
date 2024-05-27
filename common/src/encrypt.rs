use crate::error_code::BackendError;
use crate::error_code::BackendRes;
use anyhow::Ok;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;
//use ed25519_dalek::Signer;
use anyhow::Result;
use bs58;
use ed25519_dalek::Signer as DalekSigner;
use ed25519_dalek::Verifier;
use hex::ToHex;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

pub fn bs58_to_hex(input: &str) -> Result<String> {
    let decoded = bs58::decode(input.as_bytes()).into_vec()?;
    Ok(decoded.encode_hex())
}

fn hex_to_bs58(input: &str) -> Result<String> {
    let bytes = hex::decode(input)?;
    Ok(bs58::encode(&bytes).into_string())
}

fn ed25519_sign_bytes(prikey_hex: &str, data: &[u8]) -> Result<String> {
    let prikey_bytes = hex::decode(prikey_hex)?;
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes)?;
    let sig = secret_key.sign(data);
    Ok(sig.to_string())
}

pub fn ed25519_sign_hex(prikey_hex: &str, data: &str) -> Result<String> {
    let bytes: Vec<u8> = hex::decode(data)?;
    ed25519_sign_bytes(prikey_hex, &bytes)
}

//no case use it now
pub fn ed25519_sign_raw(prikey_hex: &str, data: &str) -> Result<String> {
    let bytes: Vec<u8> = data.as_bytes().to_vec();
    ed25519_sign_bytes(prikey_hex, &bytes)
}

//pubkey+real_sig
pub fn ed25519_gen_pubkey_sign(prikey_hex: &str, data_hex: &str) -> Result<String> {
    let prikey_bytes = hex::decode(prikey_hex)?;
    let data = hex::decode(data_hex)?;
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes)?;
    let sig = secret_key.sign(&data);
    let pubkey = hex::encode(secret_key.public.as_bytes());
    let pub_sig = format!("{}{}", pubkey, sig);
    Ok(pub_sig)
}

pub fn ed25519_key_gen() -> (String, String) {
    let mut csprng = OsRng {};
    let key_pair = ed25519_dalek::Keypair::generate(&mut csprng);
    let prikey: String = key_pair.secret.encode_hex();
    let pubkey: String = key_pair.public.to_bytes().encode_hex();
    let prikey = format!("{}{}", prikey, pubkey);
    (prikey, pubkey)
}

pub fn ed25519_verify_raw(data: &str, pubkey_hex: &str, sig: &str) -> Result<bool> {
    let public_key_bytes: Vec<u8> = hex::decode(pubkey_hex)?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)?;
    let signature = ed25519_dalek::Signature::from_str(sig)?;
    if public_key.verify(data.as_bytes(), &signature).is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn ed25519_verify_hex(data: &str, pubkey_hex: &str, sig: &str) -> Result<bool> {
    let public_key_bytes: Vec<u8> = hex::decode(pubkey_hex)?;
    let data: Vec<u8> = hex::decode(data)?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)?;
    let signature = ed25519_dalek::Signature::from_str(sig)?;
    if public_key.verify(data.as_slice(), &signature).is_ok() {
        Ok(true)
    } else {
        Ok(false)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::ed25519::signature::Signature;
    use near_crypto::{ED25519SecretKey, PublicKey};

    #[test]
    fn test_ed25519_gen_pubkey_sign() {
        let prikey = "d4b1b6b824f7ce4651df65a9071ad0363388675a09bbec646d4a3a1d40b67fe8";
        let data = "79a30e8d88df32b8e2d89f1467fb0f238e28f2070c9fb16f64343522a3fa77f0045e27c63bc1895ac3d36bf5a5c9d01d2e22ac40ee1d84d822123b1819947b0c";
        let hex = ed25519_gen_pubkey_sign(prikey, data).unwrap();
        println!("____{}", hex);
        assert_eq!(
            hex,
            "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9"
        );
    }

    #[test]
    fn test_bs58_to_hex() {
        let data = "24eeXypYZLjg4oUGhtqZ8BUaiBvsUQKXE6HV5tkj7yWx";
        let hex = bs58_to_hex(data).unwrap();
        assert_eq!(
            hex,
            "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9"
        );
    }

    #[test]
    fn test_hex_to_bs58() {
        let data = "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9";
        let hex = hex_to_bs58(data).unwrap();
        assert_eq!(hex, "24eeXypYZLjg4oUGhtqZ8BUaiBvsUQKXE6HV5tkj7yWx");
    }

    #[test]
    fn test_sign() {
        let (prikey, pubkey) = ed25519_key_gen();
        let input_hex = "hello";
        let sig = ed25519_sign_raw(&prikey, input_hex).unwrap();
        let verify_res = ed25519_verify_raw(input_hex, &pubkey, &sig).unwrap();
        assert!(verify_res);
    }
}
