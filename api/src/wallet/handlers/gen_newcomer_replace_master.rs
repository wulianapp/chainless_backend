use actix_web::error::InternalError;
use actix_web::{web, HttpRequest};
use common::error_code::BackendRes;
use models::secret_store::SecretStoreView;
//use log::info;
use crate::utils::captcha::{Captcha, ContactType, Usage};
use crate::utils::token_auth;
use crate::wallet::{CreateMainAccountRequest, GenTxNewcomerReplaceMasterRequest, ReconfirmSendMoneyRequest};
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


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenReplaceKeyInfo {
    pub add_key_txid: String,
    pub add_key_raw: String,
    pub delete_key_txid: String,
    pub delete_key_raw: String,
}
pub(crate) async fn req(
    req: HttpRequest,
    request_data: GenTxNewcomerReplaceMasterRequest,
) -> BackendRes<GenReplaceKeyInfo> {
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let GenTxNewcomerReplaceMasterRequest {
        newcomer_pubkey,
    } = request_data;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    let main_account = user_info.user_info.main_account;
    super::have_no_uncompleted_tx(&main_account)?;


    let client = ContractClient::<MultiSig>::new();
    let master_list = client.get_master_pubkey_list(&main_account).await;

    if master_list.len() != 1 {
        error!("unnormal account， it's account have more than 1 master");
        return Err(common::error_code::BackendError::InternalError("".to_string()));
    }
    let master = master_list.first().unwrap();
    
    let (add_key_txid,add_key_raw) = client.add_key(&main_account, &newcomer_pubkey).await.unwrap().unwrap();
    let (delete_key_txid,delete_key_raw) = client.delete_key(&main_account, &master).await.unwrap().unwrap();
    let replace_txids = GenReplaceKeyInfo{
        add_key_txid,
        add_key_raw,
        delete_key_txid,
        delete_key_raw
    };


    /*** 
    //增加之前判断是否有
    if !master_list.contains(&newcomer_pubkey.to_string()){
        let res = client.add_key(&main_account, &newcomer_pubkey).await.unwrap().unwrap();
        /*** 
        let signature = crate::multi_sig::ed25519_sign_data2(
            master_prikey,
            &res.0,
        );
        let test = crate::general::broadcast_tx_commit_from_raw2(&res.1,&signature).await;
        */
    }else{
        error!("newcomer_pubkey<{}> already is master",newcomer_pubkey);
    }

    //删除之前判断目标新公钥是否在，在的话就把新公钥之外的全删了
    let mut master_list = client.get_master_pubkey_list(&main_account).await;
    master_list.retain(|x| x != &newcomer_pubkey);
    if !master_list.is_empty(){
        //理论上生产环境other_master不会超过1
        if master_list.len() == 1 {
            let other_master = master_list.first().unwrap();
            let res = client.delete_key(&main_account, &other_master).await.unwrap().unwrap();
        }else {
            error!("other master more than 1");
        }
    }else{
        error!("main account have no other master");
    }

    info!("new wallet {:?}  successfully", user_info);
    **/
    
    Ok(Some(replace_txids))
}
