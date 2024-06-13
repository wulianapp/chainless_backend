use actix_web::HttpRequest;

use crate::utils::{get_user_context, token_auth};
use common::data_structures::KeyRole;
use common::error_code::BackendRes;

use models::device_info::{DeviceInfoEntity, DeviceInfoFilter, DeviceInfoUpdater};

use models::PsqlOp;

pub(crate) async fn req(req: HttpRequest) -> BackendRes<String> {
    //todo: must be called by main device

    let (user_id, _, device_id, _) = token_auth::validate_credentials(&req).await?;

    let context = get_user_context(&user_id, &device_id).await?;
    let role = context.role()?;

    super::check_role(role, KeyRole::Servant)?;

    DeviceInfoEntity::update(
        DeviceInfoUpdater::HolderSaved(true),
        DeviceInfoFilter::ByDeviceUser(&device_id, &user_id),
    )
    .await?;
    Ok(None::<String>)
}
