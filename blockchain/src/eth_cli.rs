use common::{
    error_code::{BackendError, ExternalServiceError},
};
use ethers::{middleware::SignerMiddleware, providers::JsonRpcError};
use lazy_static::lazy_static;
use near_jsonrpc_client::{methods, JsonRpcClient, MethodCallResult};
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use common::error_code;
use near_crypto::{InMemorySigner, PublicKey, Signer};
use near_primitives::{
    account::{AccessKey, AccessKeyPermission},
    borsh::{self, BorshSerialize},
    transaction::{
        Action, AddKeyAction, CreateAccountAction, DeleteKeyAction, FunctionCallAction,
        SignedTransaction, Transaction, TransferAction,
    },
    types::{AccountId, BlockReference, Finality, FunctionArgs},
    views::{FinalExecutionStatus, QueryRequest},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use std::{hash::Hash, marker::PhantomData, str::FromStr};
use tracing::{debug, error, info};
use anyhow::{Result};

use ethers::types::transaction::eip712::Eip712;
use ethers::{abi::Address, types::Signature};
use ethers_contract::{Eip712, EthAbiType};
use ethers_core::{k256::ecdsa::SigningKey, types::U256};
use ethers_signers::{LocalWallet};

use ethers::prelude::*;
use std::sync::Arc;

use crate::general::{gen_transaction, gen_transaction_with_caller_with_nonce};

lazy_static! {
    //static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://123.56.252.201:8061");
    static ref CHAIN_CLIENT: JsonRpcClient = {
        println!("+++__{}",common::env::CONF.chain_rpc);
        JsonRpcClient::connect(&common::env::CONF.chain_rpc)
    };
    //e05eb9eb3223d310252755e1c2fd65d03a3f9b45955186b4bea78c292cdcaa2b
    //cb5afaa026d3de65de0ddcfb1a464be8960e334a
}


#[derive(Clone)]
pub struct EthContractClient<E> {
    pub client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    pub contract_addr: H160,
    pub phantom: PhantomData<E>,
}