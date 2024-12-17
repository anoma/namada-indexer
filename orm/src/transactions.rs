use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use shared::transaction::{
    InnerTransaction, TransactionExitStatus, TransactionKind,
    WrapperTransaction,
};

use crate::schema::{inner_transactions, wrapper_transactions};

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionKind"]
pub enum TransactionKindDb {
    TransparentTransfer,
    ShieldedTransfer,
    ShieldingTransfer,
    UnshieldingTransfer,
    IbcMsgTransfer,
    Bond,
    Redelegation,
    Unbond,
    Withdraw,
    ClaimRewards,
    VoteProposal,
    InitProposal,
    ChangeMetadata,
    ChangeCommission,
    RevealPk,
    BecomeValidator,
    ReactivateValidator,
    DeactivateValidator,
    UnjailValidator,
    Unknown,
}

impl From<TransactionKind> for TransactionKindDb {
    fn from(value: TransactionKind) -> Self {
        match value {
            TransactionKind::TransparentTransfer(_) => {
                Self::TransparentTransfer
            }
            TransactionKind::ShieldedTransfer(_) => Self::ShieldedTransfer,
            TransactionKind::IbcMsgTransfer(_) => Self::IbcMsgTransfer,
            TransactionKind::Bond(_) => Self::Bond,
            TransactionKind::Redelegation(_) => Self::Redelegation,
            TransactionKind::Unbond(_) => Self::Unbond,
            TransactionKind::Withdraw(_) => Self::Withdraw,
            TransactionKind::ClaimRewards(_) => Self::ClaimRewards,
            TransactionKind::ProposalVote(_) => Self::VoteProposal,
            TransactionKind::InitProposal(_) => Self::InitProposal,
            TransactionKind::MetadataChange(_) => Self::ChangeMetadata,
            TransactionKind::CommissionChange(_) => Self::ChangeCommission,
            TransactionKind::DeactivateValidator(_) => {
                Self::DeactivateValidator
            }
            TransactionKind::ReactivateValidator(_) => {
                Self::ReactivateValidator
            }
            TransactionKind::RevealPk(_) => Self::RevealPk,
            TransactionKind::BecomeValidator(_) => Self::BecomeValidator,
            TransactionKind::UnjailValidator(_) => Self::UnjailValidator,
            TransactionKind::Unknown(_) => TransactionKindDb::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionResult"]
pub enum TransactionResultDb {
    Applied,
    Rejected,
}

impl From<TransactionExitStatus> for TransactionResultDb {
    fn from(value: TransactionExitStatus) -> Self {
        match value {
            TransactionExitStatus::Applied => TransactionResultDb::Applied,
            TransactionExitStatus::Rejected => TransactionResultDb::Rejected,
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = inner_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InnerTransactionInsertDb {
    pub id: String,
    pub wrapper_id: String,
    pub kind: TransactionKindDb,
    pub data: Option<String>,
    pub memo: Option<String>,
    pub exit_code: TransactionResultDb,
}

pub type InnerTransactionDb = InnerTransactionInsertDb;

impl InnerTransactionInsertDb {
    pub fn from(tx: InnerTransaction) -> Self {
        Self {
            id: tx.tx_id.to_string(),
            wrapper_id: tx.wrapper_id.to_string(),
            kind: TransactionKindDb::from(tx.kind),
            data: tx.data,
            memo: tx.memo,
            exit_code: TransactionResultDb::from(tx.exit_code),
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = wrapper_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WrapperTransactionInsertDb {
    pub id: String,
    pub fee_payer: String,
    pub fee_token: String,
    pub gas_limit: String,
    pub gas_used: Option<String>,
    pub block_height: i32,
    pub exit_code: TransactionResultDb,
    pub atomic: bool,
}

pub type WrapperTransactionDb = WrapperTransactionInsertDb;

impl WrapperTransactionInsertDb {
    pub fn from(tx: WrapperTransaction) -> Self {
        Self {
            id: tx.tx_id.to_string(),
            fee_payer: tx.fee.gas_payer.to_string(),
            fee_token: tx.fee.gas_token.to_string(),
            gas_limit: tx.fee.gas,
            gas_used: tx.fee.gas_used,
            block_height: tx.block_height as i32,
            exit_code: TransactionResultDb::from(tx.exit_code),
            atomic: tx.atomic,
        }
    }
}
