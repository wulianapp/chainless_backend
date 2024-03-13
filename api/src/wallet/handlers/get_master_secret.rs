use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{account_manager::{UserFilter, UserInfoView}, secret_store::{SecretFilter, SecretStoreView}, PsqlOp};

use crate::{utils::token_auth};
use common::{data_structures::secret_store::SecretStore, error_code::BackendRes};
use serde::{Deserialize, Serialize};

use crate::wallet::GetStrategyRequest;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyDataTmp {
    pub multi_sig_ranks: Vec<MultiSigRank>,
    pub master_pubkey: String,
    pub servant_pubkeys: Vec<String>,
    pub subaccounts: Vec<String>,
}

pub(crate) async fn req(
    req: HttpRequest
) -> BackendRes<SecretStore> {
    let (user_id,device_id,_) = token_auth::validate_credentials2(&req)?;

    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;
    let cli = blockchain::ContractClient::<MultiSig>::new();
    //todo： 保证只有更换主设备的瞬间才会出现两个，其他都是1个master，或者在底层加一个替换的接口
    let master_key = cli.get_master_pubkey(&main_account).await;
    let secret =  SecretStoreView::find_single(SecretFilter::ByPubkey(master_key))?;
    Ok(Some(secret.secret_store))
}
