use orm::transactions::{
    InnerTransactionDb, TransactionKindDb, TransactionResultDb,
    WrapperTransactionDb,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TransactionResult {
    Applied,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionKind {
    TransparentTransfer,
    ShieldedTransfer,
    ShieldingTransfer,
    UnshieldingTransfer,
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
    IbcMsgTransfer,
    BecomeValidator,
    DeactivateValidator,
    ReactivateValidator,
    UnjailValidator,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WrapperTransaction {
    pub tx_id: String,
    pub fee_payer: String,
    pub fee_token: String,
    pub gas_limit: String,
    pub gas_used: Option<String>,
    pub block_height: u64,
    pub inner_transactions: Vec<ShortInnerTransaction>,
    pub exit_code: TransactionResult,
    pub atomic: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortInnerTransaction {
    pub tx_id: String,
    pub kind: TransactionKind,
    pub data: Option<String>,
    pub memo: Option<String>,
    pub exit_code: TransactionResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerTransaction {
    pub tx_id: String,
    pub wrapper_id: String,
    pub kind: TransactionKind,
    pub data: Option<String>,
    pub memo: Option<String>,
    pub exit_code: TransactionResult,
}

impl InnerTransaction {
    pub fn to_short(&self) -> ShortInnerTransaction {
        ShortInnerTransaction {
            tx_id: self.tx_id.clone(),
            kind: self.kind.clone(),
            data: self.data.clone(),
            memo: self.memo.clone(),
            exit_code: self.exit_code.clone(),
        }
    }
}

impl From<TransactionResultDb> for TransactionResult {
    fn from(value: TransactionResultDb) -> Self {
        match value {
            TransactionResultDb::Applied => TransactionResult::Applied,
            TransactionResultDb::Rejected => TransactionResult::Rejected,
        }
    }
}

impl From<TransactionKindDb> for TransactionKind {
    fn from(value: TransactionKindDb) -> Self {
        match value {
            TransactionKindDb::TransparentTransfer => Self::TransparentTransfer,
            TransactionKindDb::ShieldedTransfer => Self::ShieldedTransfer,
            TransactionKindDb::ShieldingTransfer => Self::ShieldingTransfer,
            TransactionKindDb::UnshieldingTransfer => Self::UnshieldingTransfer,
            TransactionKindDb::Bond => Self::Bond,
            TransactionKindDb::Redelegation => Self::Redelegation,
            TransactionKindDb::Unbond => Self::Unbond,
            TransactionKindDb::Withdraw => Self::Withdraw,
            TransactionKindDb::ClaimRewards => Self::ClaimRewards,
            TransactionKindDb::VoteProposal => Self::VoteProposal,
            TransactionKindDb::InitProposal => Self::InitProposal,
            TransactionKindDb::ChangeMetadata => Self::ChangeMetadata,
            TransactionKindDb::ChangeCommission => Self::ChangeCommission,
            TransactionKindDb::RevealPk => Self::RevealPk,
            TransactionKindDb::Unknown => Self::Unknown,
            TransactionKindDb::IbcMsgTransfer => Self::IbcMsgTransfer,
            TransactionKindDb::BecomeValidator => Self::BecomeValidator,
            TransactionKindDb::ReactivateValidator => Self::ReactivateValidator,
            TransactionKindDb::DeactivateValidator => Self::DeactivateValidator,
            TransactionKindDb::UnjailValidator => Self::UnjailValidator,
        }
    }
}

impl From<WrapperTransactionDb> for WrapperTransaction {
    fn from(value: WrapperTransactionDb) -> Self {
        Self {
            tx_id: value.id,
            fee_payer: value.fee_payer,
            fee_token: value.fee_token,
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            block_height: value.block_height as u64,
            inner_transactions: vec![],
            exit_code: TransactionResult::from(value.exit_code),
            atomic: value.atomic,
        }
    }
}

impl From<InnerTransactionDb> for InnerTransaction {
    fn from(value: InnerTransactionDb) -> Self {
        Self {
            tx_id: value.id,
            wrapper_id: value.wrapper_id,
            kind: TransactionKind::from(value.kind),
            data: value.data,
            memo: value.memo,
            exit_code: TransactionResult::from(value.exit_code),
        }
    }
}
