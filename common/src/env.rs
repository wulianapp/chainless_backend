use std::sync::{Mutex, MutexGuard};
use std::{env, fmt};

use std::fmt::Debug;
use std::str::FromStr;

use tracing::info;
use tracing_futures::Instrument;

use crate::utils::time::MINUTE30;

#[derive(Debug, PartialEq)]
pub struct Relayer {
    pub pri_key: String,
    pub account_id: String,
}


#[derive(Debug, PartialEq)]
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

///read config data for env
#[derive(Debug)]
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
    /// ws servie prot
    pub multi_sig_relayer_prikey: String,
    /// relayer_account_id
    pub multi_sig_relayer_account_id: String,
    /// psql connect url
    pub wallet_api_port: usize,
    pub captcha_valid_interval: u64,
    pub login_by_password_retry_time: u64,
    /// redis
    pub prostgres_server: String,
    /// eth rpc url
    pub chain_rpc: String,
    /// chain id
    pub stmp_account: String,
    /// chain ws url
    pub stmp_sender: String,
    ///  main address
    pub stmp_password: String,
    ///  stroage contract address
    pub stmp_server: String,
    ///  token proxy contract address
    pub stmp_port: usize,
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

impl Default for EnvConf {
    fn default() -> Self {
        EnvConf {
            service_mode: ServiceMode::Test,
            api_port: 8065,
            multi_sig_contract: "".to_string(),
            multi_sig_relayer_prikey: "".to_string(),
            multi_sig_relayer_account_id: "".to_string(),
            wallet_api_port: 8069,
            captcha_valid_interval: MINUTE30,
            prostgres_server: "".to_string(),
            chain_rpc: "1".to_string(),
            stmp_account: "1".to_string(),
            stmp_sender: "1".to_string(),
            stmp_password: "1".to_string(),
            stmp_server: "1".to_string(),
            stmp_port: 8064,
            sms_server: "1".to_string(),
            sms_account: "1".to_string(),
            sms_token: "1".to_string(),
            fees_call_contract: "fees_call".to_string(),
            bridge_near_contract: "cvault0001.chainless".to_string(),
            bridge_eth_contract: "0x1234".to_string(),
            bridge_admin_prikey: "".to_string(),
            eth_wbtc_contract: "0x1234".to_string(),
            eth_usdt_contract: "0x1234".to_string(),
            eth_usdc_contract: "0x1234".to_string(),
            eth_dw20_contract: "0x1234".to_string(),
            eth_cly_contract: "0x1234".to_string(),
            login_by_password_retry_time: 5,
            btc_aggregated_api_base_uri: String::new()
        }
    }
}

lazy_static! {
    //业务模块具体处理是否必须从环境变量注入
    pub static ref CONF: EnvConf = {
        //fix: no default
        let mut conf = EnvConf::default();

        //todo:don't repeat your self
        if let Some(value) = env::var_os("BACKEND_SERVICE_MODE"){
            conf.service_mode = ServiceMode::from_str(value.to_str().unwrap()).unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_API_PORT"){
            conf.api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_MULTI_SIG_CONTRACT"){
            conf.multi_sig_contract = value.to_str().unwrap().parse().unwrap();
        }


        if let Some(value) = env::var_os("BACKEND_FEES_CALL_CONTRACT"){
            conf.fees_call_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_BRIDGE_NEAR_CONTRACT"){
            conf.bridge_near_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_BRIDGE_ETH_CONTRACT"){
            conf.bridge_eth_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_BRIDGE_ADMIN_PRIKEY"){
            conf.bridge_admin_prikey = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_ERC20_USDT_CONTRACT"){
            conf.eth_usdt_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_ERC20_USDC_CONTRACT"){
            conf.eth_usdc_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_ERC20_DW20_CONTRACT"){
            conf.eth_dw20_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_ERC20_WBTC_CONTRACT"){
            conf.eth_wbtc_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_ERC20_CLY_CONTRACT"){
            conf.eth_cly_contract = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_MULTI_SIG_RELAYER_PRIKEY"){
            conf.multi_sig_relayer_prikey = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_MULTI_SIG_RELAYER_ACCOUNT_ID"){
            conf.multi_sig_relayer_account_id = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_CHAIN_RPC"){
            println!("__{}",value.to_str().unwrap());
            conf.chain_rpc = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_GENERAL_API_PORT"){
            conf.multi_sig_relayer_prikey = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_WALLET_API_PORT"){
            conf.wallet_api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_CAPTCHA_VALID_INTERVAL"){
            conf.captcha_valid_interval = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_PORT"){
            println!("__{:?}",value.to_str().unwrap().to_string());
            conf.stmp_port  = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_POSTGRES_SERVER"){
            conf.prostgres_server = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_POSTGRES_SERVER"){
            conf.prostgres_server = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_ACCOUNT"){
            conf.stmp_account = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_SENDER"){
            conf.stmp_sender = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_PASSWORD"){
            conf.stmp_password = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_SERVER"){
            conf.stmp_server = value.to_str().unwrap().to_owned();
        }

         if let Some(value) = env::var_os("BACKEND_SMS_SERVER"){
            conf.sms_server = value.to_str().unwrap().to_owned();
        }

         if let Some(value) = env::var_os("BACKEND_SMS_ACCOUNT"){
            conf.sms_account = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMS_TOKEN"){
            conf.sms_token = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("BACKEND_SMTP_PORT"){
            println!("__{:?}",value.to_str().unwrap().to_string());
            conf.stmp_port  = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("BACKEND_BTC_AGGREGATED_API_BASE_URI"){
            conf.btc_aggregated_api_base_uri  = value.to_str().unwrap().parse().unwrap();
        }

        conf
    };
    pub static ref TOKEN_SECRET_KEY: String = {
        if let Some(value) = env::var_os("TOKEN_SECRET_KEY"){
            value.to_str().unwrap().parse().unwrap()
        }else{
            "your_secret_key".to_string()
        }
    };

    pub static ref DB_SERVICE: String = {
        if let Some(value) = env::var_os("DB_SERVICE"){
            value.to_str().unwrap().parse().unwrap()
        }else{
            "host=localhost user=postgres port=8068 password=postgres".to_string()
        }
    };

    //use relayer array to avoid nonce conflict
    pub static ref MULTI_SIG_RELAYER_POOL: Vec<Mutex<Relayer>> = {
        //BACKEND_MULTI_SIG_RELAYER_PRIKEY
        //BACKEND_MULTI_SIG_RELAYER_ACCOUNT_ID
        let pri_key: String = env::var_os("BACKEND_MULTI_SIG_RELAYER_PRIKEY")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let account_id:String = env::var_os("BACKEND_MULTI_SIG_RELAYER_ACCOUNT_ID")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let relayer = Mutex::new(Relayer{
            pri_key,
            account_id
        });
        vec![relayer]
    };
}

pub fn find_idle_relayer() -> Option<&'static Mutex<Relayer>>{
    for relayer in MULTI_SIG_RELAYER_POOL.iter() {
        match relayer.try_lock() {
            Ok(_) => {
                return Some(relayer);
            },
            Err(_) => continue,
        }
    }
    None
}

pub async fn wait_for_idle_relayer() -> &'static Mutex<Relayer>{
    loop {
        match find_idle_relayer(){
            Some(x) => return x,
            None => {
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                continue;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_env() {
        println!("envs {:?}", *super::CONF);
    }
}
