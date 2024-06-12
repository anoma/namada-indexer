use orm::gas::GasDb;
use serde::{Deserialize, Serialize};

use super::transaction::TransactionKind;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gas {
    pub token: String,
    pub gas_limit: u64,
    pub tx_kind: TransactionKind,
}

impl From<GasDb> for Gas {
    fn from(value: GasDb) -> Self {
        Self {
            token: value.token,
            gas_limit: value.gas_limit as u64,
            tx_kind: TransactionKind::from(value.tx_kind),
        }
    }
}
