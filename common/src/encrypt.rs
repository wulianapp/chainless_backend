
use crate::data_structures::wallet::get_support_coin_list;
use crate::error_code::BackendError;
use crate::error_code::BackendRes;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use anyhow::Ok;
use tracing::error;
//use ed25519_dalek::Signer;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use rand::rngs::OsRng;
use serde_json::json;
use bs58;
use serde::{Deserialize, Serialize};
use tracing::debug;
use anyhow::Result;
use ed25519_dalek::Verifier;


pub fn bs58_to_hex(input: &str) -> Result<String> {
    let decoded = bs58::decode(
        input.as_bytes()
    ).into_vec()?;
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
    ed25519_sign_bytes(prikey_hex,&bytes)
}

//no case use it now
pub fn ed25519_sign_raw(prikey_hex: &str, data: &str) -> Result<String> {
    let bytes: Vec<u8> = data.as_bytes().to_vec();
    ed25519_sign_bytes(prikey_hex,&bytes)
}

//pubkey+real_sig
pub fn ed25519_gen_pubkey_sign(prikey_hex: &str, data_hex: &str) -> Result<String> {
    let prikey_bytes = hex::decode(prikey_hex)?;
    let data = hex::decode(data_hex)?;
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes)?;
    let sig = secret_key.sign(&data);
    let pubkey = hex::encode(secret_key.public.as_bytes());
    let pub_sig = format!("{}{}",pubkey,sig);
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

pub fn ed25519_verify(data:&str,pubkey_hex:&str,sig:&str) -> Result<bool> {
    let public_key_bytes: Vec<u8> = hex::decode(pubkey_hex)?;
    let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)?;
    let signature = ed25519_dalek::Signature::from_str(sig)?;
    if public_key.verify(data.as_bytes(), &signature).is_ok(){
        Ok(true)
    }else {   
        Ok(false)
    }
}

/***
pub fn sign_data_by_near_wallet2(prikey_str: &str, data_str: &str) -> String {
    let prikey: SecretKey = prikey_str.parse().unwrap();
    let prikey_bytes = prikey.unwrap_as_ed25519().0;
    let data = hex::decode(data_str).unwrap();

    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string());
    let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
    let signer = InMemorySigner::from_secret_key(signer_account_id, near_secret);
    let signature = signer.sign(&data);
    let near_sig_bytes = signature.try_to_vec().unwrap();
    let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
    hex::encode(ed25519_sig_bytes)
}

pub fn sign_data_by_near_wallet(prikey_bytes: [u8; 64], data: &[u8]) -> String {
    let near_secret: SecretKey = SecretKey::ED25519(ED25519SecretKey(prikey_bytes));
    let main_device_pubkey = get_pubkey(&near_secret.to_string());
    let signer_account_id = AccountId::from_str(&main_device_pubkey).unwrap();
    let signer = InMemorySigner::from_secret_key(signer_account_id, near_secret);
    let signature = signer.sign(data);
    let near_sig_bytes = signature.try_to_vec().unwrap();
    let ed25519_sig_bytes = near_sig_bytes.as_slice()[1..].to_vec();
    hex::encode(ed25519_sig_bytes)
}
*/
#[cfg(test)]
mod tests {
    use ed25519_dalek::ed25519::signature::Signature;
    use near_crypto::{ED25519SecretKey, PublicKey};
    use super::*;

    #[test]
    fn test_bs58_to_hex(){
        let data = "24eeXypYZLjg4oUGhtqZ8BUaiBvsUQKXE6HV5tkj7yWx";
        let hex = bs58_to_hex(data).unwrap();
        assert_eq!(hex,"0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9");
    }

    #[test]
    fn test_hex_to_bs58(){
        let data = "0fcaff42a5dada720c865dcf0589413559447d361dd307f17aac1a2679944ad9";
        let hex = hex_to_bs58(data).unwrap();
        assert_eq!(hex,"24eeXypYZLjg4oUGhtqZ8BUaiBvsUQKXE6HV5tkj7yWx");
    }

    #[test]
    fn test_sign(){
        let (prikey,pubkey) = ed25519_key_gen();
        let input_hex = "hello";
        let sig = ed25519_sign_raw(&prikey, input_hex).unwrap();
        let verify_res = ed25519_verify(input_hex,&pubkey,&sig).unwrap();
        assert!(verify_res);
    }
   
}