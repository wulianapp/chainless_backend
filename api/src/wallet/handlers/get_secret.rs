use actix_web::HttpRequest;

use blockchain::multi_sig::{MultiSig, MultiSigRank, StrategyData};
use models::{secret_store::{SecretFilter, SecretStoreView}, PsqlOp};

use crate::{utils::token_auth, wallet::GetSecretRequest};
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
    req: HttpRequest,
    request_data: GetSecretRequest,
) -> BackendRes<SecretStore> {
    let _user_id = token_auth::validate_credentials(&req)?;
    let pubkey = request_data.pubkey;
    let secret =
     SecretStoreView::find_single(SecretFilter::ByPubkey(pubkey))?;
    Ok(Some(secret.secret_store))
}
