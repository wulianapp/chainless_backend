use actix_web::HttpRequest;

use blockchain::multi_sig::MultiSig;
use common::data_structures::wallet_namage_record::WalletOperateType;
use models::general::{get_pg_pool_connect, transaction_begin};
use models::wallet_manage_record::WalletManageRecordView;

use crate::utils::token_auth;
use common::data_structures::{KeyRole2, SecretKeyState, SecretKeyType};
use common::error_code::BackendRes;
use common::error_code::{AccountManagerError, WalletError};
use models::account_manager::{UserFilter, UserInfoView};
use models::device_info::{DeviceInfoFilter, DeviceInfoUpdater, DeviceInfoView};
use models::secret_store::{SecretFilter, SecretUpdater};

use crate::wallet::{AddServantRequest, RemoveServantRequest};
use blockchain::ContractClient;
use common::error_code::BackendError::{self, InternalError};
use models::secret_store::SecretStoreView;
use models::{PgLocalCli, PsqlOp};
use tracing::error;

pub(crate) async fn req(
    req: HttpRequest,
    request_data: RemoveServantRequest,
) -> BackendRes<String> {
    //todo: must be called by main device
    let (user_id, device_id, device_brand) = token_auth::validate_credentials2(&req)?;
    let RemoveServantRequest { servant_pubkey } = request_data;
    let mut pg_cli: PgLocalCli = get_pg_pool_connect().await?;
    let mut pg_cli =  pg_cli.begin().await?;

    let (user, mut current_strategy, device) =
        super::get_session_state(user_id, &device_id,&mut pg_cli).await?;
    let main_account = user.main_account;
    super::have_no_uncompleted_tx(&main_account,&mut pg_cli).await?;
    let current_role = super::get_role(&current_strategy, device.hold_pubkey.as_deref());
    super::check_role(current_role, KeyRole2::Master)?;
    

    //old key_store set abandoned
    SecretStoreView::update_single(
        SecretUpdater::State(SecretKeyState::Abandoned),
        SecretFilter::ByPubkey(&servant_pubkey),
        &mut pg_cli
    ).await?;

    //待添加的设备一定是已经登陆的设备，如果是绕过前端直接调用则就直接报错
    DeviceInfoView::update_single(
        DeviceInfoUpdater::BecomeUndefined(&servant_pubkey),
        DeviceInfoFilter::ByHoldKey(&servant_pubkey),
        &mut pg_cli
    ).await?;

    //add wallet info
    let cli = ContractClient::<MultiSig>::new().await?;
    current_strategy
        .servant_pubkeys
        .retain(|x| x != &servant_pubkey);
    let tx_id = cli
        .update_servant_pubkey(&main_account, current_strategy.servant_pubkeys)
        .await?;


    //todo: generate txid before call contract
    let record = WalletManageRecordView::new_with_specified(
        &user_id.to_string(),
        WalletOperateType::RemoveServant,
        &current_strategy.master_pubkey,
        &device_id,
        &device_brand,
        vec![tx_id],
    );
    record.insert(&mut pg_cli).await?;
    pg_cli.commit().await?;
    Ok(None::<String>)
}
