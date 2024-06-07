use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use common::env::CONF;
use reqwest;
use rust_decimal::Decimal;
use serde::Deserialize;

pub struct AccountSummary {
    pub days: i64,
    pub balance: Decimal,
}

#[derive(Deserialize)]
struct Data {
    block: i64,
    time: i64,
    balance: String,
}

#[derive(Deserialize)]
struct Body {
    status: u8,
    message: Option<String>,
    result: Option<Data>,
}

pub async fn first_tx(account: &str) -> Result<Option<AccountSummary>> {
    let body = reqwest::get(format!(
        "{}/address/first_tx/{}",
        CONF.btc_aggregated_api_base_uri, account
    ))
    .await?
    .json::<Body>()
    .await?;

    let res = match (body.status, body.result) {
        (1, Some(data)) if data.time <= 0 => None,
        (1, Some(data)) => {
            let dt = DateTime::from_timestamp(data.time, 0).ok_or(anyhow!("invalid timestamp"))?;
            let days = (Utc::now() - dt).num_days();
            let balance = Decimal::from_str(data.balance.as_str())?;
            Some(AccountSummary { days, balance })
        }
        (_, _) => None,
    };
    Ok(res)
}
