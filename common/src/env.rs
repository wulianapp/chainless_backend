use std::{env, fmt};

use std::fmt::Debug;
use std::str::FromStr;

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
    ///http service port
    pub account_manage_api_port: usize,
    pub airdrop_api_port: usize,
    /// ws servie prot
    pub general_api_port: usize,
    /// psql connect url
    pub wallet_api_port: usize,
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
    ///pri key for settlement
    pub sms_account: String,
    ///bot key
    pub sms_token: String,
}

impl Default for EnvConf {
    fn default() -> Self {
        EnvConf {
            service_mode: ServiceMode::Local,
            account_manage_api_port: 8065,
            airdrop_api_port: 8067,
            general_api_port: 8068,
            wallet_api_port: 8069,
            prostgres_server: "host=localhost user=postgres port=5432 password=postgres"
                .to_string(),
            chain_rpc: "".to_string(),
            stmp_account: "".to_string(),
            stmp_sender: "".to_string(),
            stmp_password: "".to_string(),
            stmp_server: "".to_string(),
            stmp_port: 8064,
            sms_server: "".to_string(),
            sms_account: "".to_string(),
            sms_token: "".to_string(),
        }
    }
}

lazy_static! {
    //业务模块具体处理是否必须从环境变量注入
    pub static ref CONF: EnvConf = {
        let mut conf = EnvConf::default();

        //todo:don't repeat your self
        if let Some(value) = env::var_os("SERVICE_MODE"){
            conf.service_mode = ServiceMode::from_str(value.to_str().unwrap()).unwrap();
        }

        if let Some(value) = env::var_os("ACCOUNT_MANAGER_API_PORT"){
            conf.account_manage_api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("AIRDROP_API_PORT"){
            conf.airdrop_api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("GENERAL_API_PORT"){
            conf.general_api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("WALLET_API_PORT"){
            conf.wallet_api_port = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("SMTP_PORT"){
            conf.stmp_port  = value.to_str().unwrap().parse().unwrap();
        }

        if let Some(value) = env::var_os("POSTGRES_SERVER"){
            conf.prostgres_server = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("POSTGRES_SERVER"){
            conf.chain_rpc = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("SMTP_ACCOUNT"){
            conf.stmp_account = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("SMTP_SENDER"){
            conf.stmp_sender = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("SMTP_PASSWORD"){
            conf.stmp_password = value.to_str().unwrap().to_owned();
        }

        if let Some(value) = env::var_os("SMTP_SERVER"){
            conf.stmp_server = value.to_str().unwrap().to_owned();
        }

         if let Some(value) = env::var_os("SMS_SERVER"){
            conf.sms_server = value.to_str().unwrap().to_owned();
        }

         if let Some(value) = env::var_os("SMS_ACCOUNT"){
            conf.sms_account = value.to_str().unwrap().to_owned();
        }

         if let Some(value) = env::var_os("SMS_TOKEN"){
            conf.sms_token = value.to_str().unwrap().to_owned();
        }

        conf
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_env() {
        println!("envs {:?}", *super::CONF);
    }
}
