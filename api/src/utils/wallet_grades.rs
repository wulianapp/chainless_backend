use std::str::FromStr;

use anyhow::{anyhow, Result};
use rust_decimal::Decimal;

use super::btc_aggregated_api::{first_tx, AccountSummary};

lazy_static! {
    static ref WALLET_GRADES: [(u8, Decimal); 6] = [
        // 金牌
        (1, Decimal::from_str("50").unwrap()),
        // 银牌
        (2, Decimal::from_str("25").unwrap()),
        // 铜牌
        (3, Decimal::from_str("12.5").unwrap()),
        // 铁牌
        (4, Decimal::from_str("6.25").unwrap()),
        // 锡牌
        (5, Decimal::from_str("3.125").unwrap()),
        // 纸牌
        (6, Decimal::from_str("0.5").unwrap()),
    ];
}

pub fn calc_wallet_score(account: Option<AccountSummary>) -> Decimal {
    match account {
        Some(a) if a.days >= 180 => {
            Decimal::from(a.days / 360)
                + Decimal::from(a.days % 360 / 30) * Decimal::from_str("0.08").unwrap()
                + a.balance
        }
        _ => Decimal::ZERO,
    }
}

pub fn calc_wallet_grade(score: Decimal) -> u8 {
    for r in WALLET_GRADES.into_iter() {
        if score >= r.1 {
            return r.0;
        }
    }
    0
}

pub async fn query_wallet_grade(account: &str) -> Result<u8> {
    let data = first_tx(account).await?;
    let score = calc_wallet_score(data);
    Ok(calc_wallet_grade(score))
}
