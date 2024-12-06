use orm::pgf::{PaymentKindDb, PaymentRecurrenceDb};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentRecurrence {
    Retro,
    Continuous,
}

impl From<PaymentRecurrenceDb> for PaymentRecurrence {
    fn from(value: PaymentRecurrenceDb) -> Self {
        match value {
            PaymentRecurrenceDb::Continuous => Self::Continuous,
            PaymentRecurrenceDb::Retro => Self::Retro,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentKind {
    Native,
    Ibc,
}

impl From<PaymentKindDb> for PaymentKind {
    fn from(value: PaymentKindDb) -> Self {
        match value {
            PaymentKindDb::Ibc => Self::Ibc,
            PaymentKindDb::Native => Self::Native,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PgfPayment {
    pub payment_recurrence: PaymentRecurrence,
    pub proposal_id: i32,
    pub payment_kind: PaymentKind,
    pub receipient: String,
    pub amount: String,
}
