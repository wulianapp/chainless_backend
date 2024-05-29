use crate::error_code::BackendError;
use crate::error_code::BackendRes;
use anyhow::Ok;
use anyhow::Result;
use bitcoin::address::Address;
use bitcoin::ecdsa::Signature;
use bitcoin::hashes::sha256;
use bitcoin::hashes::Hash;
use bitcoin::key;
use bitcoin::key::PublicKey;
use bitcoin::network::Network;
use bitcoin::opcodes;
use bitcoin::script;
use bitcoin::CompressedPublicKey;
use bitcoin::XOnlyPublicKey;
use bs58;
use hex::ToHex;
use rand::rngs::OsRng;
use rand::Rng;
use secp256k1::ecdsa::RecoverableSignature;
use secp256k1::ecdsa::RecoveryId;
use secp256k1::Message;
use secp256k1::{Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::debug;
use tracing::error;

pub fn calculate_p2wpkh_address(hex_key: &str) -> Result<String> {
    let key_bytes = hex::decode(hex_key)?;
    let key = CompressedPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2wpkh(&key, Network::Testnet);
    Ok(addr.to_string())
}

pub fn calculate_p2shwpkh_address(hex_key: &str) -> Result<String> {
    let key_bytes = hex::decode(hex_key)?;
    let key = CompressedPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2shwpkh(&key, Network::Testnet);
    Ok(addr.to_string())
}

pub fn calculate_p2tr_address(hex_key: &str) -> Result<String> {
    /*** 
    let key_bytes = hex::decode(hex_key)?;
    let secp = Secp256k1::new();
    let key = XOnlyPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2tr(&secp, key, None, Network::Testnet);
    Ok(addr.to_string())
    ***/
    todo!()
}

pub fn calculate_p2pkh_address(hex_key: &str) -> Result<String> {
    /***
    let key_bytes = hex::decode(hex_key)?;
    let key = CompressedPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2pkh(&key, Network::Testnet);
    Ok(addr.to_string())
    ***/
    todo!()
}

pub fn new_secret_key() -> Result<(String, String)> {
    let secp = Secp256k1::new();
    let mut value = rand::thread_rng();
    let value: [u8; 32] = value.gen();
    let prikey = SecretKey::from_slice(value.as_slice())?;
    let prikey_str = prikey.display_secret().to_string();
    let pubkey_str = prikey.public_key(&secp).to_string();
    Ok((prikey_str, pubkey_str))
}

pub fn new_p2tr_secret_key() -> Result<(String, String)> {
    let (prikey,_) = new_secret_key()?;
    /*** 
    //let sk = hex::decode(prikey)?;
    let sk = SecretKey::from_str(&prikey)?;
    let script_pubkey = script::Builder::new()
            .push_opcode(opcodes::all::OP_1ADD)
            .push_slice(sk.into())
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();
    Ok((prikey, script_pubkey.to_hex_string()))
    **/
    todo!()
}

pub fn verify(data: &str, sig: &str, address: &str) -> Result<bool> {
    let data = hex::decode(data)?;
    let message_hash = sha256::Hash::const_hash(&data);
    let msg = Message::from_digest_slice(&message_hash.to_byte_array())?;
    let signature_bytes = hex::decode(sig)?;
    let (recovery_id_byte, signature_bytes) = signature_bytes.split_at(1);
    println!("recovery_id_byte {:?}", recovery_id_byte);
    let recovery_id = RecoveryId::from_i32(recovery_id_byte[0] as i32)?;
    //let signature = Signature::from_compact(signature_bytes).expect("compact signature");
    let secp = Secp256k1::new();
    let signature = RecoverableSignature::from_compact(&signature_bytes, recovery_id)?;
    let recovered_pubkey = secp.recover_ecdsa(&msg, &signature)?;

    if address == calculate_p2wpkh_address(&recovered_pubkey.to_string())?
        || address == calculate_p2shwpkh_address(&recovered_pubkey.to_string())?
        || address == calculate_p2tr_address(&recovered_pubkey.to_string())?
        || address == calculate_p2pkh_address(&recovered_pubkey.to_string())?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn sign(sk: &str, data: &str) -> Result<String> {
    let data = hex::decode(data)?;
    let message_hash = sha256::Hash::const_hash(&data);
    let msg = Message::from_digest_slice(&message_hash.to_byte_array())?;
    let secp = Secp256k1::new();
    let sk = SecretKey::from_str(sk)?;
    let (recover_id, mut recover_sig) = secp.sign_ecdsa_recoverable(&msg, &sk).serialize_compact();
    let mut sig = vec![recover_id.to_i32() as u8];
    sig.append(&mut recover_sig.to_vec());
    Ok(hex::encode(sig))
}

#[cfg(test)]
mod tests {
    use crate::prelude::CHAINLESS_AIRDROP;

    use super::*;
    #[test]
    fn test_specp256k1_sign_verify() {
        let user_id = 123;
        let data = format!("{}_{}",user_id,CHAINLESS_AIRDROP);
        let raw_data = hex::encode(data.as_bytes());
        let (prikey, pubkey) = new_secret_key().unwrap();
        let sig = sign(&prikey, &raw_data).unwrap();
        println!("prikey {},pubkey {},sig {}", prikey, pubkey, sig);
        let address = calculate_p2wpkh_address(&pubkey).unwrap();
        let address = calculate_p2shwpkh_address(&pubkey).unwrap();
        //let address = calculate_p2tr_address(&pubkey).unwrap();
        //let address = calculate_p2pkh_address(&pubkey).unwrap();
        println!("p2tr_address {}", address);
        let res = verify(&raw_data, &sig, &address).unwrap();
        assert!(res);
    }
}
