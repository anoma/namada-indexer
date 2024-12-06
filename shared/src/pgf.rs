use serde::Serialize;

use crate::balance::Amount;
use crate::id::Id;

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum PaymentRecurrence {
    Continuous,
    Retro,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum PaymentKind {
    Ibc,
    Native,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum PgfAction {
    Add,
    Remove,
}

#[derive(Debug, Clone)]
pub struct PgfPayment {
    pub proposal_id: u64,
    pub recurrence: PaymentRecurrence,
    pub kind: PaymentKind,
    pub receipient: Id,
    pub amount: Amount,
    pub action: Option<PgfAction>,
}
