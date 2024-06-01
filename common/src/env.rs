//use std::sync::{Mutex, MutexGuard};
use std::{env, fmt, fs};
use near_primitives::types::Nonce;
use tokio::sync::{Mutex, MutexGuard};

use std::fmt::Debug;
use std::str::FromStr;

use serde::Deserialize;
use tracing::{debug, info, warn};
use tracing_futures::Instrument;

use crate::utils::time::MINUTE30;

#[derive(Deserialize, Debug, PartialEq, EnumString, Display)]
pub enum ServiceMode {
    Product,
    Dev,
    Local,
    Test, //for testcase
}

#[derive(Deserialize, Debug)]
pub struct Database {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl Database {
    pub fn db_uri(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.dbname
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct Smtp {
    pub server: String,
    pub sender: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Sms {
    pub cmtelecom_api_key: String,
    pub smsbao_username: String,
    pub smsbao_api_key: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct RelayerPool {
    pub seed: String,
    pub account_id: String,
    pub derive_size: u16,
}

///read config data for env
#[derive(Deserialize, Debug)]
pub struct EnvConf {
    /// dev or pro
    pub service_mode: ServiceMode,
    /// http service port
    pub api_port: usize,
    pub multi_sig_contract: String,
    pub fees_call_contract: String,
    pub bridge_near_contract: String,
    pub bridge_eth_contract: String,
    pub bridge_admin_prikey: String,
    pub relayer_pool: RelayerPool,
    /// psql connect url
    pub wallet_api_port: usize,
    pub captcha_valid_interval: u64,
    pub login_by_password_retry_time: u64,
    pub database: Database,
    /// eth rpc url
    pub chain_rpc: String,
    pub stmp: Smtp,
    pub sms: Sms,
    pub eth_wbtc_contract: String,
    pub eth_usdt_contract: String,
    pub eth_usdc_contract: String,
    pub eth_dw20_contract: String,
    pub eth_cly_contract: String,
    /// BTC aggregated API service
    pub btc_aggregated_api_base_uri: String,
    pub error_code_path: String,
}

lazy_static! {
    pub static ref CONF: EnvConf = {
        let content= fs::read_to_string(env::var_os("CONFIG").expect("CONFIG environment variable required"))
            .expect("Unable to read the `CONFIG` specified file");
        toml::from_str(content.as_str()).expect("contents of configuration file invalid")
    };

    pub static ref TOKEN_SECRET_KEY: String = {
        if let Some(value) = env::var_os("TOKEN_SECRET_KEY"){
            value.to_str().unwrap().parse().unwrap()
        }else{
            "your_secret_key".to_string()
        }
    };
}

#[cfg(test)]
mod tests {
    use tracing::error;

    use crate::log::init_logger;

    use super::*;
    #[test]
    fn test_get_env() {
        println!("envs {:?}", *super::CONF);
    }
}
