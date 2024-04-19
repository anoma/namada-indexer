use diesel::data_types::PgNumeric;
use diesel::Insertable;

use crate::schema::nam_balances;
use crate::utils::{Base10000BigUint, PgNumericInt};
use shared::balance::Balance;

#[derive(Insertable, Clone)]
#[diesel(table_name = nam_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NamBalancesInsertDb {
    pub owner: String,
    pub raw_amount: PgNumeric,
}

impl From<Balance> for NamBalancesInsertDb {
    fn from(value: Balance) -> Self {
        let num = Base10000BigUint::from(value.amount);
        let raw_amount = PgNumericInt::from(num);

        Self {
            owner: value.owner.to_string(),
            raw_amount: raw_amount.into_inner(),
        }
    }
}
