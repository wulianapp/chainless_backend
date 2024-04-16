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

pub mod coin_amount{
    use std::ops::{Div, Mul};
    pub const BASE_DECIMAL: u128 = 1_000_000_000_000_000_000;//18
    pub const DISPLAY_DECIMAL:  u128 = 1_00_000_000;//8
    pub const DEDUCT_DECIMAL:  u128 = 10_000_000_000;//10

    pub fn raw2display(raw:u128) ->String {
        //截取后方的10位
        let raw = raw / DEDUCT_DECIMAL;
        let dist = format!("{}.{:08}",raw / 100_000_000,raw % 10_000_000);
        dist
    }
    
    pub fn display2raw(display:&str) -> u128 {
        let split_res:Vec<&str> = display.split('.').collect();
        if split_res.len() != 2 || split_res[1].len() != 8{
            panic!("");
        }


        let integer_part = u128::from_str_radix(split_res[0], 10).unwrap();
        let point_part = u128::from_str_radix(split_res[1], 10).unwrap();
        integer_part * BASE_DECIMAL + point_part * DEDUCT_DECIMAL
    }
}




//生成随机值的hex字符串
pub fn generate_random_hex_string(size: usize) -> String {
    // 计算需要生成的随机字节数
    let byte_size = (size + 1) / 2;

    // 生成随机字节数组
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; byte_size];
    rng.fill(&mut bytes[..]);

    let hex_string = hex::encode(&bytes);
    hex_string.chars().take(size).collect()
}

#[cfg(test)]
mod tests {
    use super::coin_amount::*;
    #[test]
    fn test_generate_random_hex_string() {

        let value  = super::generate_random_hex_string(64);
        print!("value {}_",value);

        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_coin_amount(){
        assert_eq!(raw2display(1u128 * BASE_DECIMAL),"1.00000000".to_string());
        assert_eq!(raw2display(1u128 * BASE_DECIMAL + 100u128 * DEDUCT_DECIMAL),"1.00000100".to_string());
        assert_eq!(raw2display(110u128 * DEDUCT_DECIMAL),"0.00000110".to_string());
        assert_eq!(raw2display(123u128),"0.00000000".to_string());


        assert_eq!(display2raw("100.00010000"),100u128 * BASE_DECIMAL + 10000u128 * DEDUCT_DECIMAL);
        //assert_eq!(display2raw("0.00"),0u128);
        assert_eq!(display2raw("112.00000001"),112u128 * BASE_DECIMAL + 1u128 * DEDUCT_DECIMAL);
        //assert_eq!(display2raw("112.0000000000000000000001"),100u128 * BASE_DECIMAL);

    }
}
