use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::balance::Balance;

use crate::schema::balances;

#[derive(Insertable, Clone, Queryable, Selectable, Debug)]
#[diesel(table_name = balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalancesInsertDb {
    pub owner: String,
    pub token: String,
    pub raw_amount: BigDecimal,
}

pub type BalanceDb = BalancesInsertDb;

impl BalancesInsertDb {
    pub fn from_balance(balance: Balance) -> Self {
        let asd = Self {
            owner: balance.owner.to_string(),
            token: balance.token.to_string(),
            raw_amount: BigDecimal::from_str(&balance.amount.to_string())
                .expect("Invalid amount"),
        };

        println!(
            "Balance_test: {:?}",
            BigDecimal::from_str(&balance.amount.to_string())
        );

        asd
    }
}
