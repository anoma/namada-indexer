use crate::{balance::Amount, id::Id};

#[derive(Debug, Clone)]
pub struct MaspEntry {
    pub token_address: String,
    pub timestamp: i64,
    pub raw_amount: Amount,
    pub inner_tx_id: Id,
}