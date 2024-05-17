use std::sync::{Mutex, MutexGuard};
use std::{env, fmt, fs};

use std::fmt::Debug;
use std::str::FromStr;

use serde::Deserialize;
use tracing::{info, warn};
use tracing_futures::Instrument;

use crate::utils::time::MINUTE30;

pub struct Relayer {
    pub pri_key: String,
    pub account_id: String,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct RelayerPool {
    pub pri_key: String,
    pub base_account_id: String,
    pub derive_size: u16
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ServiceMode {
    Product,
    Dev,
    Local,
    Test, //for testcase
}

impl std::str::FromStr for ServiceMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "product" => Ok(ServiceMode::Product),
            "dev" => Ok(ServiceMode::Dev),
            "local" => Ok(ServiceMode::Local),
            "test" => Ok(ServiceMode::Test),
            _ => Err("Don't support this service mode".to_string()), // 处理未知字符串的情况
        }
    }
}

impl fmt::Display for ServiceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            ServiceMode::Product => "product",
            ServiceMode::Dev => "dev",
            ServiceMode::Local => "local",
            ServiceMode::Test => "test",
        };
        write!(f, "{}", description)
    }
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
    ///  vault contract address
    pub sms_server: String,
    /// pri key for settlement
    pub sms_account: String,
    /// bot key
    pub sms_token: String,
    pub eth_wbtc_contract: String,
    pub eth_usdt_contract: String,
    pub eth_usdc_contract: String,
    pub eth_dw20_contract: String,
    pub eth_cly_contract: String,
    /// BTC aggregated API service
    pub btc_aggregated_api_base_uri: String,
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

    //use relayer array to avoid nonce conflict
    pub static ref MULTI_SIG_RELAYER_POOL: Vec<Mutex<Relayer>> = {
        let RelayerPool { pri_key, base_account_id, derive_size }
            = CONF.relayer_pool.clone();
        let mut pool = vec![Mutex::new(Relayer{
            pri_key: pri_key.clone(),
            account_id: base_account_id.clone()
        })];
        for index in (0..derive_size).into_iter() {
            pool.push(Mutex::new(Relayer{
                pri_key: pri_key.clone(),
                account_id: format!("{}{}",base_account_id,index)
            })); 
        }
        pool    
    };
}

pub fn find_idle_relayer() -> Option<&'static Mutex<Relayer>> {
    for relayer in MULTI_SIG_RELAYER_POOL.iter() {
        match relayer.try_lock() {
            Ok(_) => {
                return Some(relayer);
            }
            Err(_) => continue,
        }
    }
    None
}

pub async fn wait_for_idle_relayer() -> &'static Mutex<Relayer> {
    loop {
        match find_idle_relayer() {
            Some(x) => {
                return x;
            }
            None => {
                warn!("relayer is busy");
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_env() {
        println!("envs {:?}", *super::CONF);
    }

    #[tokio::test]
    async fn test_relayer_pool() {
        let mut handles = vec![];
        for index in 0..1000 {
            let handle = tokio::spawn(async move {
                let relayer = wait_for_idle_relayer().await;
                //println!("envs {:?} index {}", relayer.lock().unwrap(),index);
                index
            });
            handles.push(handle);
        }
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        assert_eq!(results, (0..1000).into_iter().collect::<Vec<_>>());
    }
}
