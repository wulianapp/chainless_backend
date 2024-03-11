use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::error_code::BackendRes;
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CommitNewcomerReplaceMasterRequest, CreateMainAccountRequest, GenTxNewcomerReplaceMasterRequest, ReconfirmSendMoneyRequest};
use blockchain::multi_sig::MultiSig;
use blockchain::ContractClient;
use common::data_structures::account_manager::UserInfo;
use common::data_structures::secret_store::SecretStore;
use common::error_code::AccountManagerError::{
    InviteCodeNotExist, PhoneOrEmailAlreadyRegister, PhoneOrEmailNotRegister,
};
use models::account_manager::{get_next_uid, UserFilter, UserInfoView, UserUpdater};
use models::{account_manager, secret_store, PsqlOp};
use tracing::{error, info};
use serde::{Deserialize,Serialize};


pub(crate) async fn req(
    req: HttpRequest,
    request_data: CommitNewcomerReplaceMasterRequest,
) -> BackendRes<String> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let CommitNewcomerReplaceMasterRequest {
        newcomer_pubkey,
        add_key_raw,
        delete_key_raw,
        add_key_sig,
        delete_key_sig,
        newcomer_prikey_encrypted_by_pwd,
        newcomer_prikey_encrypted_by_answer,
    } = request_data;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;

    let client = ContractClient::<MultiSig>::new();
    let master_list = client.get_master_pubkey_list(&main_account).await;

    if master_list.len() != 1 {
        error!("unnormal account， it's account have more than 1 master");
        return Err(common::error_code::BackendError::InternalError("".to_string()));
    }
    let master = master_list.first().unwrap();

    //增加之前判断是否有
    if !master_list.contains(&newcomer_pubkey.to_string()){
        blockchain::general::broadcast_tx_commit_from_raw2(&add_key_raw,&add_key_sig).await;
    }else{
        error!("newcomer_pubkey<{}> already is master",newcomer_pubkey);
    }

    //删除之前判断目标新公钥是否在，在的话就把新公钥之外的全删了
    let mut master_list = client.get_master_pubkey_list(&main_account).await;
    master_list.retain(|x| x != &newcomer_pubkey);
    if !master_list.is_empty(){
        //理论上生产环境other_master不会超过1
        if master_list.len() == 1 {
            blockchain::general::broadcast_tx_commit_from_raw2(&delete_key_raw,&delete_key_sig).await;
        }else {
            error!("other master more than 1");
        }
    }else{
        error!("main account have no other master");
    }

    //info!("new wallet {:?}  successfully", user_info);
    
    
    Ok(None::<String>)
}
