use anyhow::anyhow;
use anyhow::Result;
use blockchain::multi_sig::MultiSig;
use blockchain::multi_sig::StrategyData;
use blockchain::ContractClient;
use common::{data_structures::{account_manager::UserInfo, device_info::DeviceInfo, KeyRole2}, error_code::{BackendError, WalletError}};
use models::{
  account_manager::{UserFilter, UserInfoEntity},
  coin_transfer::{CoinTxEntity, CoinTxFilter},
  device_info::{DeviceInfoEntity, DeviceInfoFilter},
  PgLocalCli, PsqlOp,
};

pub mod api_test;
pub mod btc_aggregated_api;
pub mod captcha;
pub mod respond;
pub mod token_auth;
pub mod wallet_grades;

//通过合约数据来判定设备角色
pub fn judge_role_by_strategy(strategy: Option<&StrategyData>,device_key:Option<&str>) -> Result<KeyRole2>{
  let role = match (strategy,device_key) {
    (None, None) => KeyRole2::Undefined,
    (None, Some(_)) =>   Err(anyhow!("unreachable"))?,
    (Some(_), None) => KeyRole2::Undefined,
    (Some(strategy), Some(hold_key)) => {
        if strategy.master_pubkey.eq(hold_key){
            KeyRole2::Master
        }else if strategy.servant_pubkeys.contains(&hold_key.to_string()){
            KeyRole2::Servant
        }else {
            //如果从设备被删之后，就变成了新设备，但此时设备表仍留存之前该从设备的信息
            //Err(anyhow!("unknown key {}",hold_key))?
            KeyRole2::Undefined
        }
    }
  };
  Ok(role)
}

pub async fn judge_role_by_account(device_key:Option<&str>,account: &str) -> Result<KeyRole2>{
  let cli = ContractClient::<MultiSig>::new_query_cli().await?;
  let strategy = cli.get_strategy(&account).await?;
  judge_role_by_strategy(strategy.as_ref(),device_key)
}

pub async fn judge_role_by_user_id(device_key:Option<&str>,id: &u32,db_cli:&mut PgLocalCli<'_>) -> Result<KeyRole2>{
  let user_info = UserInfoEntity::find_single(UserFilter::ById(id), db_cli)
  .await
  .map_err(|err| {
      if err.to_string().contains("DBError::DataNotFound") {
          WalletError::MainAccountNotExist(err.to_string()).into()
      } else {
          BackendError::InternalError(err.to_string())
      }
  })?.into_inner();

  if let Some(ref account) = user_info.main_account {
    judge_role_by_account(device_key,account).await
  }else{
    Ok(KeyRole2::Undefined)
  }
}

//all state info
pub struct UserContext{
  pub user_info: UserInfo,
  pub device: DeviceInfo,
  pub strategy: Option<StrategyData>,
}
impl UserContext{
  pub fn role(&self) -> Result<KeyRole2>{
    judge_role_by_strategy(
      self.strategy.as_ref(),
      self.device.hold_pubkey.as_deref()
    )
  }

  pub fn account_strategy(&self) -> Result<(String,StrategyData),WalletError> {
    //main_acocunt和strategy是同时有或者同时无
    let strategy = self.strategy.clone().ok_or(WalletError::NotSetSecurity)?;
    Ok((self.user_info.clone().main_account.unwrap(),strategy))
  }
}

//获取当前会话的已进行安全问答的用户信息、多签配置、设备信息的属性数据
pub async fn get_user_context(
  user_id: &u32,
  device_id: &str,
  conn: &mut PgLocalCli<'_>,
) -> Result<UserContext, BackendError> {
  let user_info = UserInfoEntity::find_single(UserFilter::ById(&user_id), conn)
      .await
      .map_err(|err| {
          if err.to_string().contains("DBError::DataNotFound") {
              WalletError::MainAccountNotExist(err.to_string()).into()
          } else {
              BackendError::InternalError(err.to_string())
          }
      })?.into_inner();

    //注册过的一定有设备信息
  let device = DeviceInfoEntity::find_single(
    DeviceInfoFilter::ByDeviceUser(device_id, &user_id),
     conn)
        .await?.into_inner();

  let strategy = match user_info.main_account {
      Some(ref account) => {
        let multi_sig_cli = ContractClient::<MultiSig>::new_query_cli().await?;
        multi_sig_cli.get_strategy(account).await?
      },
      None => None,
  };

  Ok(UserContext{user_info,device,strategy})
}

