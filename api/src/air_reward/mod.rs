
#![feature(async_closure)]
//! account manager http service
pub mod handlers;

use actix_web::{get, post, web, HttpRequest, Responder};

use blockchain::{air_reward::SysInfo, bridge_on_near};
use blockchain::bridge_on_near::Status;
use common::data_structures::{
    bridge::{DepositStatus, WithdrawStatus},
    CoinType,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, Level};

//use captcha::{ContactType, VerificationCode};

use crate::utils::respond::gen_extra_respond;
use crate::utils::respond::get_lang;
use common::log::generate_trace_id;

/**
* @api {get} /airReward/getSysInfo 获取合约公共信息
* @apiVersion 0.0.1
* @apiName LetSysInfo
* @apiGroup AirReward
* @apiHeader {String} Authorization  user's access token
* @apiExample {curl} Example usage:
*   curl -X POST http://120.232.251.101:8066/wallet/getStrategy
* -H "Content-Type: application/json" -H 'Authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGci
   OiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxLCJkZXZpY2VfaWQiOiIyIiwiaWF0IjoxNzA2ODQ1ODgwODI3LCJleHA
   iOjE3MDgxNDE4ODA4Mjd9.YsI4I9xKj_y-91Cbg6KtrszmRxSAZJIWM7fPK7fFlq8'
* @apiSuccess {String=0,1,3011} status_code         状态码.
* @apiSuccess {String} msg 状态信息
* @apiSuccess {Object} data                          订单列表详情.
* @apiSuccess {Object} data.net_users        不关注
* @apiSuccess {Number} data.net_users.user_receive_dw20       不关注
* @apiSuccess {Number} data.net_users.user_receive_cly        不关注
* @apiSuccess {String} data.admin            管理员
* @apiSuccess {String} data.admin.0            -
* @apiSuccess {String} data.admin.1            -
* @apiSuccess {Object} data.settle_times       3,9,21,top排行榜已结算时间
* @apiSuccess {Number} data.settle_times.three       
* @apiSuccess {Number} data.settle_times.nine       
* @apiSuccess {Number} data.settle_times.twenty_one   
* @apiSuccess {Object} data.next_settle_times   下一次结算时间
* @apiSuccess {Number} data.next_settle_times.three       
* @apiSuccess {Number} data.next_settle_times.nine       
* @apiSuccess {Number} data.next_settle_times.twenty_one   
* @apiSuccess {Number} data.start_times     合约开始时间
* @apiSuccess {Number} data.fire_times      点火时间，控制开始释放时间
* @apiSuccess {Number} data.free_times      全局释放至时间
* @apiSuccess {Bool} data.free_off        仅控制释放操作是否允许
* @apiSuccess {Number} data.disuse_times    下次淘汰时间
* @apiSuccess {Number} data.times_elapsed  用于控制合约时间 用于测试
* @apiSuccess {Object[]} data.free_total_token  释放token总和
* @apiSuccess {String} data.free_total_token.0  -
* @apiSuccess {Number} data.free_total_token.1  -

* @apiSampleRequest http://120.232.251.101:8066/wallet/listWithdrawOrder
*/

type SysInfoResponse = SysInfo;
#[tracing::instrument(skip_all,fields(trace_id = generate_trace_id()))]
#[get("/airReward/getSysInfo")]
async fn get_sys_info(
    req: HttpRequest
) -> impl Responder {
    gen_extra_respond(
        get_lang(&req),
        handlers::get_sys_info::req(req).await,
    )
}

//todo: 空投卡的条件
//主设备
//kyc

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_sys_info);
}

#[cfg(test)]
mod tests {
    use crate::utils::api_test::*;
    use crate::utils::respond::BackendRespond;
    use crate::*;

    use super::*;
    use std::default::Default;
    use std::env;

    use actix_web::body::MessageBody;
    use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
    use actix_web::http::header;

    use actix_web::{body::MessageBody as _, test, App};

    use blockchain::ContractClient;
    use common::data_structures::device_info::DeviceInfo;
    use common::data_structures::KeyRole;
    use models::coin_transfer::CoinTxView;
    use models::{account_manager, secret_store, PsqlOp};
    use serde_json::json;

    use actix_web::Error;
    use blockchain::multi_sig::StrategyData;
    use blockchain::multi_sig::{CoinTx, MultiSig};
    use common::data_structures::account_manager::UserInfo;
    use common::data_structures::secret_store::SecretStore;
    use common::encrypt::ed25519_key_gen;
    use common::utils::math;
    use models::secret_store::SecretStoreView;
    // use log::{info, LevelFilter,debug,error};
    use crate::account_manager::handlers::user_info::UserInfoTmp;
    use actix_web::http::header::HeaderName;
    use actix_web::http::header::HeaderValue;
    use common::data_structures::CoinType;
    use models::account_manager::UserInfoView;
    use std::collections::HashMap;
    use tracing::{debug, error, info};

    #[actix_web::test]
    async fn test_air_reward_get_sys_info() {
        println!("start test_get_sys_info");
        let app = init().await;
        let service = test::init_service(app).await;
        let (mut sender_master, _sender_servant, _sender_newcommer, _receiver) =
            gen_some_accounts_with_new_key();

        test_register!(service, sender_master);
        test_create_main_account!(service, sender_master);
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        let sys_info = test_air_reward_get_sys_info!(service, sender_master).unwrap();
        println!("sys_info {:?}", sys_info);

    }
}
