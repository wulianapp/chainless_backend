use actix_web::{web, HttpRequest};

use blockchain::multi_sig::{MultiSig, MultiSigRank};
use common::data_structures::KeyRole2;
use models::{
    device_info::{DeviceInfoFilter, DeviceInfoView},
    PsqlOp,
};

use crate::utils::token_auth;
use crate::wallet::UpdateStrategy;
use blockchain::ContractClient;
use common::error_code::{BackendRes, WalletError};

pub async fn req(req: HttpRequest, request_data: web::Json<UpdateStrategy>) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;

    let (user,current_strategy,device) = 
    super::get_session_state(user_id,&device_id).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account)?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role,KeyRole2::Master)?;

    let UpdateStrategy {
        strategy,
    } = request_data.0;
  

    //fixme:
    let strategy = strategy
        .into_iter()
        .map(|x| MultiSigRank {
            min: x.min,
            max_eq: x.max_eq,
            sig_num: x.sig_num,
        })
        .collect();

    //add wallet info
    let cli = ContractClient::<MultiSig>::new()?;

    cli.update_rank(&main_account, strategy).await?;

    Ok(None::<String>)
}
