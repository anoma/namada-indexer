use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable, Selectable};
use shared::gas::{GasEstimation, GasPrice};

use crate::schema::{gas, gas_estimations, gas_price};
use crate::transactions::TransactionKindDb;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = gas)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasDb {
    pub tx_kind: TransactionKindDb,
    pub gas_limit: i32,
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = gas_price)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasPriceDb {
    pub token: String,
    pub amount: BigDecimal,
}

impl From<GasPrice> for GasPriceDb {
    fn from(value: GasPrice) -> Self {
        Self {
            token: value.token,
            amount: BigDecimal::from_str(&value.amount.to_string())
                .expect("Invalid amount"),
        }
    }
}

#[derive(Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = gas_estimations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GasEstimationDb {
    pub wrapper_id: String,
    pub transparent_transfer: i32,
    pub shielded_transfer: i32,
    pub shielding_transfer: i32,
    pub unshielding_transfer: i32,
    pub ibc_msg_transfer: i32,
    pub bond: i32,
    pub redelegation: i32,
    pub unbond: i32,
    pub withdraw: i32,
    pub claim_rewards: i32,
    pub vote_proposal: i32,
    pub reveal_pk: i32,
    pub signatures: i32,
    pub tx_size: i32,
}

pub type GasEstimationInsertDb = GasEstimationDb;

impl From<GasEstimation> for GasEstimationInsertDb {
    fn from(value: GasEstimation) -> Self {
        Self {
            wrapper_id: value.wrapper_id.to_string(),
            transparent_transfer: value.transparent_transfer as i32,
            shielded_transfer: value.shielded_transfer as i32,
            shielding_transfer: value.shielding_transfer as i32,
            unshielding_transfer: value.unshielding_transfer as i32,
            ibc_msg_transfer: value.ibc_msg_transfer as i32,
            bond: value.bond as i32,
            redelegation: value.redelegation as i32,
            unbond: value.unbond as i32,
            withdraw: value.withdraw as i32,
            claim_rewards: value.claim_rewards as i32,
            vote_proposal: value.vote_proposal as i32,
            reveal_pk: value.reveal_pk as i32,
            signatures: value.signatures as i32,
            tx_size: value.size as i32,
        }
    }
}
