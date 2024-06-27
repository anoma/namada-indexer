use orm::gas::{GasDb, GasPriceDb};
use serde::{Deserialize, Serialize};

use crate::service::utils::raw_amount_to_nam;

use super::transaction::TransactionKind;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gas {
    pub gas_limit: u64,
    pub tx_kind: TransactionKind,
}

impl From<GasDb> for Gas {
    fn from(value: GasDb) -> Self {
        Self {
            gas_limit: value.gas_limit as u64,
            tx_kind: TransactionKind::from(value.tx_kind),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPrice {
    pub token: String,
    pub amount: String,
}

impl From<GasPriceDb> for GasPrice {
    fn from(value: GasPriceDb) -> Self {
        Self {
            token: value.token,
            amount: raw_amount_to_nam(value.amount.to_string()),
        }
    }
}
