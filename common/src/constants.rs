use crate::utils::{
    math::BASE_DECIMAL,
    time::{DAY1, DAY7, MINUTE1, MINUTE10, MINUTE2, MINUTE30, MINUTE5},
};

/***
//验证每次请求的间隔
pub const CAPTCHA_REQUEST_INTERVAL: u64 = MINUTE1;
//验证码的有效时间
pub const CAPTCHA_EXPAIRE_TIME: u64 = MINUTE10;
//密码重试导致锁定的解锁时间
pub const LOGIN_UNLOCK_TIME: u64 = MINUTE30;
//token 有效时间
pub const TOKEN_EXPAIRE_TIME: u64 = DAY7;
//无链的链上交互基础费用
pub const MIN_BASE_FEE: u128 = 1u128 * BASE_DECIMAL;
//默认的基础交互gas数量
pub const CHAINLESS_DEFAULT_GAS_LIMIT: u64 = 600_000_000_000_000;
//交易有效时间
pub const TX_EXPAIRE_TIME: u64 = DAY1;

//充值有效时间
pub const BRIDGE_DEPOSIT_EXPIRE_TIME: u64 = MINUTE30;
**/
//验证每次请求的间隔
pub const CAPTCHA_REQUEST_INTERVAL: u64 = MINUTE1;
//验证码的有效时间
pub const CAPTCHA_EXPAIRE_TIME: u64 = MINUTE1;
//密码重试导致锁定的解锁时间
pub const LOGIN_UNLOCK_TIME: u64 = MINUTE2;
//token 有效时间
pub const TOKEN_EXPAIRE_TIME: u64 = DAY7;
//无链的链上交互基础费用
pub const MIN_BASE_FEE: u128 = BASE_DECIMAL;
//默认的基础交互gas数量
pub const CHAINLESS_DEFAULT_GAS_LIMIT: u64 = 600_000_000_000_000;
//交易有效时间
pub const TX_EXPAIRE_TIME: u64 = MINUTE5;

//充值有效时间
pub const BRIDGE_DEPOSIT_EXPIRE_TIME: u64 = MINUTE5;

// 参考币安的主网确认块数
pub const ETH_TX_CONFIRM_BLOCK_NUM: u8 = 6;

//密码登陆错误允许的重试次数
pub const LOGIN_BY_PASSWORD_RETRY_NUM: u8 = 5;

//邀请码长度规则
pub const INVITE_CODE_MIN_LEN: u8 = 4;
pub const INVITE_CODE_MAX_LEN: u8 = 20;

//btc地址验证使用的签名数据
pub const CHAINLESS_AIRDROP: &str = "ChainlessAirdrop";

