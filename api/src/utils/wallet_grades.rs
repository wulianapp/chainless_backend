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
    let data: Option<AccountSummary> = first_tx(account).await?;
    let score = calc_wallet_score(data);
    Ok(calc_wallet_grade(score))
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::data_structures::account_manager;
    use rust_decimal::prelude::FromPrimitive;

    fn calc_wallet(days: i64, balance: &str) -> (String, u8) {
        let account = AccountSummary {
            days,
            balance: Decimal::from_str(balance).unwrap(),
        };
        let score = calc_wallet_score(Some(account));
        let grade = calc_wallet_grade(score);
        println!("{},{},{},{}", days, balance, score.to_string(), grade);
        (score.to_string(), grade)
    }

    #[test]
    fn test_api_utils_wallet_grade() {
        let (score, grade) = calc_wallet(0, "100.0");
        assert_eq!((score, grade), ("0".to_string(), 0));

        let (score, grade) = calc_wallet(179, "50.0");
        assert_eq!((score, grade), ("0".to_string(), 0));

        let (score, grade) = calc_wallet(180, "49.52");
        assert_eq!((score, grade), ("50.00".to_string(), 1));

        let (score, grade) = calc_wallet(359, "49.12");
        assert_eq!((score, grade), ("50.00".to_string(), 1));

        let (score, grade) = calc_wallet(360, "2.125111");
        assert_eq!((score, grade), ("3.125111".to_string(), 5));

        let (score, grade) = calc_wallet(1110, "0.0");
        assert_eq!((score, grade), ("3.08".to_string(), 6));

        let (score, grade) = calc_wallet(1139, "0.044999");
        assert_eq!((score, grade), ("3.124999".to_string(), 6));

        let (score, grade) = calc_wallet(1135, "0.045001");
        assert_eq!((score, grade), ("3.125001".to_string(), 5));

        let (score, grade) = calc_wallet(1169, "0.0");
        assert_eq!((score, grade), ("3.16".to_string(), 5));

        let (score, grade) = calc_wallet(2250, "0.0");
        assert_eq!((score, grade), ("6.24".to_string(), 5));

        let (score, grade) = calc_wallet(2280, "0.0");
        assert_eq!((score, grade), ("6.32".to_string(), 4));

        let (score, grade) = calc_wallet(3158, "3.78001");
        assert_eq!((score, grade), ("12.50001".to_string(), 3));

        let (score, grade) = calc_wallet(3599, "15.12");
        assert_eq!((score, grade), ("25.00".to_string(), 2));

        let (score, grade) = calc_wallet(4500, "0.0");
        assert_eq!((score, grade), ("12.48".to_string(), 4));

        let (score, grade) = calc_wallet(4530, "0.0");
        assert_eq!((score, grade), ("12.56".to_string(), 3));

        let (score, grade) = calc_wallet(5578, "34.599999");
        assert_eq!((score, grade), ("49.999999".to_string(), 2));

        let (score, grade) = calc_wallet(5633, "31.5999");
        assert_eq!((score, grade), ("47.1599".to_string(), 2));
    }
}
