use base58::{FromBase58, ToBase58};
use hex::FromHex;
use rand::Rng;

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


//生成随机值的hex字符串
pub fn generate_random_hex_string(size: usize) -> String {
    // 计算需要生成的随机字节数
    let byte_size = (size + 1) / 2; // 每个十六进制字符表示 4 位二进制数，所以需要的字节数为 size / 2

    // 生成随机字节数组
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; byte_size];
    rng.fill(&mut bytes[..]);

    let hex_string = hex::encode(&bytes);
    hex_string.chars().take(size).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_random_hex_string() {

        let value  = super::generate_random_hex_string(64);
        print!("value {}_",value);

        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
