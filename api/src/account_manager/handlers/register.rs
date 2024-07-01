use blockchain::bridge_on_near::Bridge;
use blockchain::multi_sig::MultiSig;
use blockchain::register_relayer::wait_for_idle_register_relayer;
use blockchain::{admin_sign, ContractClient};
use blockchain::airdrop::Airdrop as ChainAirdrop;
use common::error_code::{AccountManagerError::*, AirdropError, BackendError, WalletError};
use common::hash::Hash;
use common::utils::math::{bs58_to_hex, random_num};
use common::data_structures::airdrop::Airdrop;
use models::airdrop::{AirdropEntity, AirdropFilter};
use models::device_info::DeviceInfoEntity;
use models::secret_store::SecretStoreEntity;
//use log::{debug, info};
use crate::utils::captcha::{Captcha, ContactType, Distinctor, Usage};

use common::error_code::BackendRes;
use models::account_manager::UserFilter;

use models::{account_manager::UserInfoEntity, PsqlOp};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    device_id: String,
    device_brand: String,
    contact: String,
    captcha: String,
    password: String,
    predecessor_invite_code: String,
    candidate_account_id: String,
    master_pubkey: String,
    master_prikey_encrypted_by_password: String,
    master_prikey_encrypted_by_answer: String,
    anwser_indexes: String,
    set_user_info_action_json: String,
}

//生成十位随机数作为user_id
const MAX_RETRY_TIMES: u8 = 10;
async fn gen_user_id() -> Result<u32, BackendError> {
    for _ in 0..MAX_RETRY_TIMES {
        let num = (random_num() % 9_000_000_000 + 1_000_000_000) as u32;
        if UserInfoEntity::find(UserFilter::ById(&num))
            .await?
            .is_empty()
        {
            return Ok(num);
        } else {
            warn!("user_id {} already exist", num);
            continue;
        }
    }
    Err(BackendError::InternalError("".to_string()))
}

pub fn gen_sign_msg(
    account_id: &str,
    phone_hash: &str,
    email_hash: &str,
    device_hash: &str,
    max_block_height: u64
) -> String{
    format!("{}-k1,k2,d1-{},{},{}-{}",
        account_id,phone_hash,email_hash,device_hash,max_block_height)
}

pub async fn req(request_data: RegisterRequest) -> BackendRes<(String,String)> {
    let RegisterRequest {
        device_id,
        device_brand,
        contact,
        captcha,
        predecessor_invite_code,
        password,
        candidate_account_id,
        master_pubkey,
        master_prikey_encrypted_by_password,
        master_prikey_encrypted_by_answer,
        anwser_indexes,
        set_user_info_action_json,
    } = request_data;

    //候选钱包id为pubkey的小写截取的长度10的字符串
    if !master_pubkey.to_lowercase().contains(&candidate_account_id)
        || candidate_account_id.len() != 10
    {
        Err(BackendError::RequestParamInvalid("".to_string()))?
    }

    let candidate_account_id = format!("{}.{}", candidate_account_id, "user");

    //check userinfo
    let user_info = UserInfoEntity::find(UserFilter::ByPhoneOrEmail(&contact)).await?;
    if !user_info.is_empty() {
        Err(PhoneOrEmailAlreadyRegister)?;
    }

    let multi_sig_cli = ContractClient::<MultiSig>::new_query_cli().await?;
    let key = multi_sig_cli
        .get_master_pubkey_list(&candidate_account_id)
        .await?;
    if !key.is_empty() {
        Err(WalletError::MainAccountAlreadyExist(
            candidate_account_id.clone(),
        ))?
    }

    Captcha::check_and_delete(&contact, &captcha, Usage::Register)?;

    //store user info
    let this_user_id = gen_user_id().await?;
    let mut view = UserInfoEntity::new_with_specified(
        this_user_id,
        &password.hash(),
        &anwser_indexes,
        &candidate_account_id,
    );
    let (phone_hash,email_hash) = match contact.contact_type()? {
        ContactType::PhoneNumber => {
            view.user_info.phone_number = Some(contact.clone());
            (contact.hash(),"".to_string())
        }
        ContactType::Email => {
            view.user_info.email = Some(contact.clone());
            ("".to_string(),contact.hash())
        }
    };
    let token_version = view.user_info.token_version;
    view.insert().await?;

    //邀请码必须存在，存在即已进行安全问答
    let predecessor_airdrop =
        AirdropEntity::find_single(AirdropFilter::ByInviteCode(&predecessor_invite_code))
            .await
            .map_err(|_e| InviteCodeNotExist)?;
    
    let Airdrop {
        user_id: predecessor_user_id,
        account_id: predecessor_account_id,
        ..
    } = predecessor_airdrop.into_inner();


    let cli = ContractClient::<ChainAirdrop>::new_query_cli().await?;
    let predecessor_airdrop_on_chain = cli
        .get_user(&predecessor_account_id)
        .await?;
    if predecessor_airdrop_on_chain.is_none() {
        Err(AirdropError::PredecessorHaveNotClaimAirdrop)?;
    }

    let user_airdrop = AirdropEntity::new_with_specified(
        this_user_id,
        &candidate_account_id,
        predecessor_user_id,
        &predecessor_account_id,
    );
    user_airdrop.insert().await?;

    let master_secret = SecretStoreEntity::new_with_specified(
        &master_pubkey,
        this_user_id,
        &master_prikey_encrypted_by_password,
        &master_prikey_encrypted_by_answer,
    );
    master_secret.insert().await?;

    let device = DeviceInfoEntity::new_with_specified(
        &device_id,
        &device_brand,
        this_user_id,
        Some(master_pubkey.clone()),
    );
    device.insert().await?;

    let allocated_relayer = wait_for_idle_register_relayer().await?;
    let msg = gen_sign_msg(
        &candidate_account_id, 
        &phone_hash, 
        &email_hash, 
    device_id.hash().as_str(), 
    allocated_relayer.busy_height.unwrap()
    );
    debug!("msg {}",msg);
    
    let sig =  admin_sign(msg.as_bytes());
    
    let token = crate::utils::token_auth::create_jwt(
        this_user_id,
        token_version,
        &device_id,
        &device_brand,
    )?;

    info!("user {} register successfully", contact);
    Ok(Some((token,sig)))
}
