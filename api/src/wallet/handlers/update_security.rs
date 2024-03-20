use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{
    account_manager::{UserFilter, UserInfoView, UserUpdater},
    device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView},
    secret_store::{SecretFilter, SecretStoreView, SecretUpdater},
    PsqlOp,
};

use crate::{
    utils::token_auth,
    wallet::{GetSecretRequest, UpdateSecurityRequest},
};
use common::{
    data_structures::{secret_store::SecretStore, KeyRole2},
    error_code::{BackendError, BackendRes, WalletError},
};
use serde::{Deserialize, Serialize};

pub(crate) async fn req(
    req: HttpRequest,
    request_data: UpdateSecurityRequest,
) -> BackendRes<String> {
    let (user_id, device_id, _) = token_auth::validate_credentials2(&req)?;
    let user_info = UserInfoView::find_single(UserFilter::ById(user_id))?;
    super::have_no_uncompleted_tx(&user_info.user_info.main_account)?;
    let device = DeviceInfoView::find_single(DeviceInfoFilter::ByDeviceUser(&device_id, user_id))?;
    if device.device_info.key_role != KeyRole2::Master {
        Err(WalletError::UneligiableRole(
            device.device_info.key_role,
            KeyRole2::Master,
        ))?;
    }

    let UpdateSecurityRequest {
        anwser_indexes,
        secrets,
    } = request_data;
    //todo: must be master
    UserInfoView::update(
        UserUpdater::AnwserIndexes(&anwser_indexes),
        UserFilter::ById(user_id),
    )?;

    for secret in secrets {
        SecretStoreView::update(
            SecretUpdater::EncrypedPrikey(
                &secret.encrypted_prikey_by_password,
                &secret.encrypted_prikey_by_answer,
            ),
            SecretFilter::ByPubkey(&secret.pubkey),
        )?;
        DeviceInfoView::update(
            DeviceInfoUpdater::HolderSaved(false),
            DeviceInfoFilter::ByHoldKey(&secret.pubkey),
        )?;
    }
    Ok(None)
}
