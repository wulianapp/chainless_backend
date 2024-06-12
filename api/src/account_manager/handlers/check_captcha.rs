use common::error_code::{AccountManagerError, BackendError, BackendRes};

use crate::utils::captcha::{Captcha, Usage};
use models::account_manager::UserFilter;

use models::{account_manager, PsqlOp};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CheckCaptchaRequest {
    contact: String,
    captcha: String,
    usage: String,
}

pub async fn req(request_data: CheckCaptchaRequest) -> BackendRes<bool> {
    let CheckCaptchaRequest {
        contact,
        captcha,
        usage,
    } = request_data;
    let kind: Usage = usage
        .parse()
        .map_err(|_err| BackendError::RequestParamInvalid("".to_string()))?;
    //todo: register can check captcha

    let check_res = match kind {
        Usage::Register => Captcha::check(&contact, &captcha, kind),
        _ => {
            let user =
                account_manager::UserInfoEntity::find_single(UserFilter::ByPhoneOrEmail(&contact))
                    .await
                    .map_err(|e| {
                        if e.to_string().contains("DBError::DataNotFound") {
                            AccountManagerError::PhoneOrEmailNotRegister.into()
                        } else {
                            BackendError::InternalError(e.to_string())
                        }
                    })?
                    .into_inner();
            Captcha::check(&user.id.to_string(), &captcha, kind)
        }
    };

    let is_ok = check_res.is_ok();
    Ok(Some(is_ok))
}
