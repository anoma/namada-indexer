use diesel::Insertable;

use crate::schema::nam_balances;
use shared::balance::Balance;

#[derive(Insertable, Clone)]
#[diesel(table_name = nam_balances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NamBalancesInsertDb {
    //TODO: change to owner
    pub address: String,
    pub amount: String,
}

impl From<Balance> for NamBalancesInsertDb {
    fn from(value: Balance) -> Self {
        Self {
            address: value.owner.to_string(),
            amount: value.amount.0,
        }
    }
}
