use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use shared::pgf::{PaymentKind, PaymentRecurrence, PgfPayment};

use crate::schema::public_good_funding;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentRecurrence"]
pub enum PaymentRecurrenceDb {
    Continuous,
    Retro,
}

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PaymentKind"]
pub enum PaymentKindDb {
    Ibc,
    Native,
}

impl From<PaymentRecurrence> for PaymentRecurrenceDb {
    fn from(value: PaymentRecurrence) -> Self {
        match value {
            PaymentRecurrence::Continuous => Self::Continuous,
            PaymentRecurrence::Retro => Self::Retro,
        }
    }
}

impl From<PaymentKind> for PaymentKindDb {
    fn from(value: PaymentKind) -> Self {
        match value {
            PaymentKind::Native => Self::Native,
            PaymentKind::Ibc => Self::Ibc,
        }
    }
}

#[derive(Insertable, Clone, Queryable, diesel::Selectable, Debug)]
#[diesel(table_name = public_good_funding)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PublicGoodFundingPaymentDb {
    pub payment_recurrence: PaymentRecurrenceDb,
    pub proposal_id: i32,
    pub payment_kind: PaymentKindDb,
    pub receipient: String,
    pub amount: BigDecimal,
}

pub type PublicGoodFundingPaymentInsertDb = PublicGoodFundingPaymentDb;

impl PublicGoodFundingPaymentInsertDb {
    pub fn from_pgf_payment(pgf_payment: PgfPayment) -> Self {
        Self {
            proposal_id: pgf_payment.proposal_id as i32,
            payment_recurrence: PaymentRecurrenceDb::from(
                pgf_payment.recurrence,
            ),
            payment_kind: PaymentKindDb::from(pgf_payment.kind),
            receipient: pgf_payment.receipient.to_string(),
            amount: BigDecimal::from_str(&pgf_payment.amount.to_string())
                .expect("Invalid amount"),
        }
    }
}
