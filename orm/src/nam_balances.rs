use std::str::FromStr;

use diesel::data_types::PgNumeric;
use diesel::Insertable;
use num_bigint::BigUint;

use crate::schema::nam_balances;
use crate::utils::{Base10000BigUint, PgNumericInt};
use shared::balance::Balance;

#[derive(Insertable, Clone)]
#[diesel(table_name = nam_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NamBalancesInsertDb {
    //TODO: change to owner
    pub address: String,
    pub raw_amount: PgNumeric,
}

impl From<Balance> for NamBalancesInsertDb {
    fn from(value: Balance) -> Self {
        let num =
            Base10000BigUint::from(BigUint::from_str(&value.amount.0).ok());
        let raw_amount = PgNumericInt::from(num);

        Self {
            address: value.owner.to_string(),
            raw_amount: raw_amount.into_inner(),
        }
    }
}
