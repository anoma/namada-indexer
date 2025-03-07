use crate::balance::Amount;
use crate::id::Id;

#[derive(Debug, Clone)]
pub enum MaspEntryDirection {
    In,
    Out,
}

#[derive(Debug, Clone)]
pub struct MaspEntry {
    pub token_address: String,
    pub timestamp: i64,
    pub raw_amount: Amount,
    pub direction: MaspEntryDirection,
    pub inner_tx_id: Id,
}
