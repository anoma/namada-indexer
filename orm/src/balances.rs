use diesel::data_types::PgNumeric;
use diesel::Insertable;
use shared::balance::Balance;
use shared::block::BlockHeight;

use crate::schema::balances;
use crate::utils::{Base10000BigUint, PgNumericInt};

#[derive(Insertable, Clone)]
#[diesel(table_name = balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalancesInsertDb {
    pub owner: String,
    pub token: String,
    pub height: i32,
    pub raw_amount: PgNumeric,
}

impl BalancesInsertDb {
    pub fn from_balance(balance: Balance, block_height: BlockHeight) -> Self {
        let num = Base10000BigUint::from(balance.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            owner: balance.owner.to_string(),
            token: balance.token.to_string(),
            height: block_height as i32,
            raw_amount: raw_amount.into_inner(),
        }
    }
}
