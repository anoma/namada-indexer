use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::balance::Balance;
use shared::token::Token;

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
        let token = match balance.token {
            Token::Native(token) => token.to_string(),
            Token::Ibc(token) => token.address.to_string(),
        };

        Self {
            owner: balance.owner.to_string(),
            token,
            raw_amount: BigDecimal::from_str(&balance.amount.to_string())
                .expect("Invalid amount"),
        }
    }
}
