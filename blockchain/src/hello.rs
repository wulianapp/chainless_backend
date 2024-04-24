use crate::general::{broadcast_tx_commit, gen_transaction};
use crate::ContractClient;
use anyhow::Result;
use near_crypto::Signer;
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use serde_json::json;

struct Hello {}

impl ContractClient<Hello> {
    pub fn new() -> Self {
        let account_id = "eddy3.node0".parse().unwrap();
        let pri_key = "ed25519:522wSBLmU2ytPz7QgcSJ9Q9Ddx811cRAkN2g5Qg5L\
            us84Bp7tjSvQzSSpaSMB72x7M3gj6yQYF9fkEScaG5agZ8N"
            .parse()
            .unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, pri_key);
        Self {
            deployed_at: "eddy1.node0".parse().unwrap(),
            relayer: signer,
            phantom: Default::default(),
        }
    }

    pub async fn query_get_greeting(&self) -> String {
        let request = methods::query::RpcQueryRequest {
            block_reference: BlockReference::Finality(Finality::Final),
            request: QueryRequest::CallFunction {
                account_id: self.deployed_at.to_owned(),
                method_name: "get_greeting".to_string(),
                args: FunctionArgs::from(json!({}).to_string().into_bytes()),
            },
        };

        let response = crate::general::call(request).await;

        if let QueryResponseKind::CallResult(result) = response.unwrap().kind {
            String::from_utf8(result.result).unwrap()
        } else {
            unreachable!()
        }
    }

    pub async fn call_set_greeting(&self, content: &str) -> Result<String> {
        let set_greeting_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
            method_name: "set_greeting".to_string(),
            args: json!({
                "greeting": content,
            })
            .to_string()
            .into_bytes(),
            gas: 100_000_000_000_000, // 100 TeraGas
            deposit: 0,
        }))];
        let mut transaction = gen_transaction(&self.relayer, &self.deployed_at.to_string()).await?;
        transaction.actions = set_greeting_actions;

        let signature = self
            .relayer
            .sign(transaction.get_hash_and_size().0.as_ref());

        let response = broadcast_tx_commit(transaction, signature).await;
        if let FinalExecutionStatus::Failure(error) = response.status {
            //Err(error.to_string())?;
            Err(anyhow::anyhow!(error.to_string()))?;
        }
        let tx_id = response.transaction.hash.to_string();
        Ok(tx_id)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_call_set_greeting() {
        let client = ContractClient::<Hello>::new();
        let res = client.call_set_greeting("test test").await.unwrap();
        println!("txid {}", res);
    }

    #[tokio::test]
    async fn test_query_get_greeting() {
        let client = ContractClient::<Hello>::new();
        let res = client.query_get_greeting().await;
        println!("res {}", res);
    }
}
