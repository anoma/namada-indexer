use orm::token::{IbcTokenDb, TokenDb};
use orm::transactions::{
    InnerTransactionDb, TransactionHistoryDb, TransactionHistoryKindDb,
    TransactionKindDb, TransactionResultDb, WrapperTransactionDb,
};
use shared::id::Id;
use shared::token::{IbcToken, Token};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TransactionKind {
    TransparentTransfer,
    ShieldedTransfer,
    ShieldingTransfer,
    UnshieldingTransfer,
    MixedTransfer,
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
    IbcTransparentTransfer,
    IbcShieldingTransfer,
    IbcUnshieldingTransfer,
    BecomeValidator,
    DeactivateValidator,
    ReactivateValidator,
    UnjailValidator,
    Unknown,
}

impl From<TransactionKindDb> for TransactionKind {
    fn from(value: TransactionKindDb) -> Self {
        match value {
            TransactionKindDb::TransparentTransfer => Self::TransparentTransfer,
            TransactionKindDb::ShieldedTransfer => Self::ShieldedTransfer,
            TransactionKindDb::ShieldingTransfer => Self::ShieldingTransfer,
            TransactionKindDb::UnshieldingTransfer => Self::UnshieldingTransfer,
            TransactionKindDb::MixedTransfer => Self::MixedTransfer,
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
            TransactionKindDb::IbcTransparentTransfer => {
                Self::IbcTransparentTransfer
            }
            TransactionKindDb::IbcShieldingTransfer => {
                Self::IbcShieldingTransfer
            }
            TransactionKindDb::IbcUnshieldingTransfer => {
                Self::IbcUnshieldingTransfer
            }
            TransactionKindDb::BecomeValidator => Self::BecomeValidator,
            TransactionKindDb::ReactivateValidator => Self::ReactivateValidator,
            TransactionKindDb::DeactivateValidator => Self::DeactivateValidator,
            TransactionKindDb::UnjailValidator => Self::UnjailValidator,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WrapperTransaction {
    pub id: Id,
    pub fee_payer: Id,
    pub fee_token: Token,
    pub gas_limit: String,
    pub gas_used: Option<u64>,
    pub amount_per_gas_unit: Option<String>,
    pub block_height: u64,
    pub exit_code: TransactionExitStatus,
    pub atomic: bool,
}

#[derive(Debug, Clone)]
pub enum TransactionExitStatus {
    Applied,
    Rejected,
}

impl From<TransactionResultDb> for TransactionExitStatus {
    fn from(value: TransactionResultDb) -> Self {
        match value {
            TransactionResultDb::Applied => Self::Applied,
            TransactionResultDb::Rejected => Self::Rejected,
        }
    }
}

impl WrapperTransaction {
    pub fn from_db(
        transaction: WrapperTransactionDb,
        tokens: Vec<(TokenDb, Option<IbcTokenDb>)>,
    ) -> Self {
        let fee_token = tokens
            .into_iter()
            .find_map(|(token, ibc_token)| {
                if transaction.fee_token == token.address {
                    match ibc_token {
                        Some(ibc_token) => Some(Token::Ibc(IbcToken {
                            address: Id::Account(ibc_token.address),
                            trace: Id::IbcTrace(ibc_token.ibc_trace),
                        })),
                        None => Some(Token::Native(Id::Account(token.address))),
                    }
                } else {
                    None
                }
            })
            .expect("Fee token should be known.");

        Self {
            id: Id::Hash(transaction.id),
            fee_payer: Id::Account(transaction.fee_payer),
            fee_token,
            gas_limit: transaction.gas_limit,
            gas_used: transaction.gas_used.map(|gas| gas as u64),
            amount_per_gas_unit: transaction.amount_per_gas_unit,
            block_height: transaction.block_height as u64,
            exit_code: TransactionExitStatus::from(transaction.exit_code),
            atomic: transaction.atomic,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerTransaction {
    pub id: Id,
    pub wrapper_id: Id,
    pub kind: TransactionKind,
    pub data: Option<String>,
    pub memo: Option<String>,
    pub exit_code: TransactionExitStatus,
}

impl From<InnerTransactionDb> for InnerTransaction {
    fn from(value: InnerTransactionDb) -> Self {
        Self {
            id: Id::Hash(value.id),
            wrapper_id: Id::Hash(value.wrapper_id),
            kind: TransactionKind::from(value.kind),
            data: value.data,
            memo: value.memo,
            exit_code: TransactionExitStatus::from(value.exit_code),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TransactionHistoryKind {
    Received,
    Sent,
}

#[derive(Clone, Debug)]
pub struct TransactionHistory {
    pub tx: InnerTransaction,
    pub target: Id,
    pub kind: TransactionHistoryKind,
    pub block_height: u64,
}

impl TransactionHistory {
    pub fn from_db(
        transaction_history_db: TransactionHistoryDb,
        inner_tx_db: InnerTransactionDb,
        block_height: i32,
    ) -> Self {
        Self {
            tx: InnerTransaction::from(inner_tx_db),
            target: Id::Account(transaction_history_db.target),
            kind: match transaction_history_db.kind {
                TransactionHistoryKindDb::Received => {
                    TransactionHistoryKind::Received
                }
                TransactionHistoryKindDb::Sent => TransactionHistoryKind::Sent,
            },
            block_height: block_height as u64,
        }
    }
}
