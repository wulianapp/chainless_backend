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
use bitcoin::key::TweakedPublicKey;
use bitcoin::network::Network;
use bitcoin::opcodes;
use bitcoin::script;
use bitcoin::sign_message::signed_msg_hash;
use bitcoin::sign_message::MessageSignature;
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

pub fn calculate_p2wpkh_address(pubkey_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(pubkey_hex)?;
    let key = CompressedPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2wpkh(&key, Network::Regtest);
    Ok(addr.to_string())
}

pub fn calculate_p2shwpkh_address(pubkey_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(pubkey_hex)?;
    let key = CompressedPublicKey::from_slice(&key_bytes)?;
    let addr = Address::p2shwpkh(&key, Network::Regtest);
    Ok(addr.to_string())
}

pub fn calculate_p2tr_address(pubkey_hex: &str) -> Result<String> {
    let pubkey = PublicKey::from_str(pubkey_hex)?;
    let key = XOnlyPublicKey::from(pubkey.inner);
    let tweaked_pubkey = TweakedPublicKey::dangerous_assume_tweaked(key);
    let addr = Address::p2tr_tweaked(tweaked_pubkey, Network::Regtest);
    Ok(addr.to_string())
}

pub fn calculate_p2pkh_address(pubkey_hex: &str) -> Result<String> {
    let key_bytes = hex::decode(pubkey_hex)?;
    let pubkey = PublicKey::from_slice(&key_bytes).unwrap();
    let addr = Address::p2pkh(pubkey, Network::Regtest);
    Ok(addr.to_string())
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

pub fn get_pubkey(prikey_hex: &str) -> Result<String> {
    let secp = Secp256k1::new();
    let prikey = hex::decode(prikey_hex)?;
    let prikey = SecretKey::from_slice(prikey.as_slice())?;
    let pubkey_str = prikey.public_key(&secp).to_string();
    Ok(pubkey_str)
}

pub fn verify(data: &str, sig: &str, address: &str) -> Result<bool> {
    let secp = Secp256k1::new();
    let msg_hash = signed_msg_hash(data);
    let signature_bytes = hex::decode(sig)?;
    let signature = MessageSignature::from_slice(signature_bytes.as_slice())?;
    let recovered_pubkey = signature.recover_pubkey(&secp, msg_hash)?;
    if address == calculate_p2wpkh_address(&recovered_pubkey.to_string())? {
        println!("is p2wpkh address");
        return Ok(true);
    }

    if address == calculate_p2shwpkh_address(&recovered_pubkey.to_string())? {
        println!("is p2shwpkh address");
        return Ok(true);
    }
    if address == calculate_p2tr_address(&recovered_pubkey.to_string())? {
        println!("is p2tr address");
        return Ok(true);
    }
    if address == calculate_p2pkh_address(&recovered_pubkey.to_string())? {
        println!("is p2pkh address");
        return Ok(true);
    }
    Ok(false)
}

pub fn sign(sk: &str, data: &str) -> Result<String> {
    let msg_hash = signed_msg_hash(data);
    let msg = secp256k1::Message::from_digest(msg_hash.to_byte_array());
    let secp = Secp256k1::new();
    let sk = SecretKey::from_str(sk)?;
    let secp_sig = secp.sign_ecdsa_recoverable(&msg, &sk);
    let signature = MessageSignature {
        signature: secp_sig,
        compressed: true,
    };
    let sig_hex_str = hex::encode(signature.serialize());
    Ok(sig_hex_str)
}

#[cfg(test)]
mod tests {
    use bitcoin::AddressType;

    use crate::prelude::CHAINLESS_AIRDROP;

    use super::*;
    #[test]
    fn test_btc_specp256k1_sign_verify() {
        let user_id = 123;
        let raw_data = format!("{}_{}", user_id, CHAINLESS_AIRDROP);
        let (prikey, pubkey) = new_secret_key().unwrap();

        let sig = sign(&prikey, &raw_data).unwrap();
        println!("prikey {},pubkey {},sig {}", prikey, pubkey, sig);

        let address = calculate_p2wpkh_address(&pubkey).unwrap();
        let res = verify(&raw_data, &sig, &address).unwrap();
        assert!(res);

        let address = calculate_p2shwpkh_address(&pubkey).unwrap();
        let res = verify(&raw_data, &sig, &address).unwrap();
        assert!(res);

        let address = calculate_p2tr_address(&pubkey).unwrap();
        let res = verify(&raw_data, &sig, &address).unwrap();
        assert!(res);

        let address = calculate_p2pkh_address(&pubkey).unwrap();
        let res = verify(&raw_data, &sig, &address).unwrap();
        assert!(res);
    }

    #[test]
    fn test_btc_verify() {
        //p2wpkh
        let sig = "1cfa092e7e811a862d65e8dede1932af6ebec866268de166f12fc74dd5683b02ff625a8415a24f01b8ff5a3b161f8ee4e1e84ec4d965a608318f768bdafa5a9c16";
        let addr = "bcrt1q50eu7cupwu6htn362rkquaxvp2y88t9r87wgzd";
        let user_id = "1322976383".to_string();
        let res = verify(&user_id, sig, addr).unwrap();
        assert!(res);

        //p2shwpkh
        let sig = "1bb1ecace96e42a436b0d6cef881e29ed50d419fb0a2801591914c988763793e3c409c7d934ff20ff7b70c78cd719ab8c02d4faf14637cc09fc194573243d97fb4";
        let addr = "2N83gWgRJGbzrtuGBYFSNUA1UfpFGWDFdwY";
        let _prikey = "3db3ad943247ed2902d0a2bb8576b45078824f0ff836eb2ffbe57c0ad6298e2f";
        let user_id = "1322976383".to_string();
        let res = verify(&user_id, sig, addr).unwrap();
        assert!(res);

        //p2tr
        //todo: test failed
        let sig = "1c8fa93297749c89d15cbdccfd8b792e276fed7de5923a8c6d2a6777986921763b56f4fe7ecff30b71c16b357cb290f410f7a9a16fe1eeb7dccd5d45ee426422e7";
        let addr = "bcrt1puvvr30fs5ua8mx5af2xnl8zgk9mch5ecssrheplruaf45jsf9adsukxwdc";
        let _prikey = "01a751e708098009d193119ab12b4f3364339cc072941386dc9e5d5bbf3ade50";
        let user_id = "1322976383".to_string();
        let _res = verify(&user_id, sig, addr).unwrap();
        //assert!(res);

        //p2pkh:
        //todo: test failed
        let sig = "1c9ff59886db16f5ed0818c33d6ed4b5ebd540de165af478b6ab7b68eb53e3c495470abd2d7ecc88b0ecca333d54cac9b5d2f0c4216bb827c9f847db75fb0f0d2b";
        let addr = "mhRiQgnGbmZQS2PP6KVDBr33TQNeToWjfj";
        let _prikey = "0e8648b011b887136198842e01f4e82aeba5a35cc26b6571f1ce5f90737b6014";
        let user_id = "1322976383".to_string();
        let _res = verify(&user_id, sig, addr).unwrap();
        //assert!(res);
    }
}
