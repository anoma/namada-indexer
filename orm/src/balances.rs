use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::balance::Balance;
use shared::block::BlockHeight;
use shared::id::Id;
use shared::pgf::PgfPayment;
use shared::token::Token;

use crate::schema::balance_changes;
use crate::views::balances;

#[derive(Insertable, Clone, Queryable, Selectable, Debug)]
#[diesel(table_name = balance_changes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalanceChangesInsertDb {
    pub owner: String,
    pub token: String,
    pub raw_amount: BigDecimal,
    pub height: i32,
}

pub type BalanceChangeDb = BalanceChangesInsertDb;

#[derive(Clone, Queryable, Selectable, Debug)]
#[diesel(table_name = balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalanceDb {
    pub owner: String,
    pub token: String,
    pub raw_amount: BigDecimal,
}

impl BalanceChangesInsertDb {
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
            height: balance.height as i32,
        }
    }

    pub fn from_pgf_retro(
        payment: PgfPayment,
        token: Id,
        block_height: BlockHeight,
    ) -> Self {
        Self {
            owner: payment.receipient.to_string(),
            height: block_height as i32,
            token: token.to_string(),
            raw_amount: BigDecimal::from_str(&payment.amount.to_string())
                .expect("Invalid amount"),
        }
    }
}
