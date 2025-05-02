use orm::transactions::WrapperTransactionDb;

use crate::appstate::AppState;
use crate::error::transaction::TransactionError;
use crate::repository::tranasaction::{
    TransactionRepository, TransactionRepositoryTrait,
};
use crate::response::transaction::{
    InnerTransaction, TransactionHistory, TransactionKind, WrapperTransaction,
};
use orm::transactions::TransactionKindDb;

#[derive(Clone)]
pub struct TransactionService {
    pub transaction_repo: TransactionRepository,
}

impl TransactionService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            transaction_repo: TransactionRepository::new(app_state),
        }
    }

    pub async fn get_wrapper_tx(
        &self,
        id: String,
    ) -> Result<Option<WrapperTransaction>, TransactionError> {
        let wrapper_tx = self
            .transaction_repo
            .find_wrapper_tx(id)
            .await
            .map_err(TransactionError::Database)?;

        let masp_fee_payment = if let Some(WrapperTransactionDb {
            masp_fee_payment: Some(masp_fee_payment),
            ..
        }) = &wrapper_tx
        {
            self.get_inner_tx(masp_fee_payment.to_owned()).await?
        } else {
            None
        };

        Ok(wrapper_tx.map(|wrapper_tx| {
            let mut wrapper = WrapperTransaction::from(wrapper_tx);
            wrapper.masp_fee_payment =
                masp_fee_payment.map(|inner| inner.to_short());
            wrapper
        }))
    }

    pub async fn get_inner_tx(
        &self,
        id: String,
    ) -> Result<Option<InnerTransaction>, TransactionError> {
        let inner_tx = self
            .transaction_repo
            .find_inner_tx(id)
            .await
            .map_err(TransactionError::Database)?;

        Ok(inner_tx.map(InnerTransaction::from))
    }

    pub async fn get_inner_tx_by_wrapper_id(
        &self,
        id: String,
    ) -> Result<Vec<InnerTransaction>, TransactionError> {
        let inner_txs = self
            .transaction_repo
            .find_inners_by_wrapper_tx(id)
            .await
            .map_err(TransactionError::Database)?;

        Ok(inner_txs.into_iter().map(InnerTransaction::from).collect())
    }

    // Helper function to parse transaction types from comma-separated string
    fn parse_transaction_types(
        transaction_types: Option<String>,
    ) -> Option<Vec<TransactionKindDb>> {
        transaction_types.map(|types_str| {
            types_str
                .split(',')
                .filter_map(|type_str| {
                    let type_str = type_str.trim();
                    match type_str {
                        "transparentTransfer" => {
                            Some(TransactionKindDb::TransparentTransfer)
                        }
                        "shieldedTransfer" => {
                            Some(TransactionKindDb::ShieldedTransfer)
                        }
                        "shieldingTransfer" => {
                            Some(TransactionKindDb::ShieldingTransfer)
                        }
                        "unshieldingTransfer" => {
                            Some(TransactionKindDb::UnshieldingTransfer)
                        }
                        "mixedTransfer" => {
                            Some(TransactionKindDb::MixedTransfer)
                        }
                        "ibcMsgTransfer" => {
                            Some(TransactionKindDb::IbcMsgTransfer)
                        }
                        "ibcTransparentTransfer" => {
                            Some(TransactionKindDb::IbcTransparentTransfer)
                        }
                        "ibcShieldingTransfer" => {
                            Some(TransactionKindDb::IbcShieldingTransfer)
                        }
                        "ibcUnshieldingTransfer" => {
                            Some(TransactionKindDb::IbcUnshieldingTransfer)
                        }
                        "bond" => Some(TransactionKindDb::Bond),
                        "redelegation" => Some(TransactionKindDb::Redelegation),
                        "unbond" => Some(TransactionKindDb::Unbond),
                        "withdraw" => Some(TransactionKindDb::Withdraw),
                        "claimRewards" => Some(TransactionKindDb::ClaimRewards),
                        "voteProposal" => Some(TransactionKindDb::VoteProposal),
                        "initProposal" => Some(TransactionKindDb::InitProposal),
                        "changeMetadata" => {
                            Some(TransactionKindDb::ChangeMetadata)
                        }
                        "changeCommission" => {
                            Some(TransactionKindDb::ChangeCommission)
                        }
                        "revealPk" => Some(TransactionKindDb::RevealPk),
                        "becomeValidator" => {
                            Some(TransactionKindDb::BecomeValidator)
                        }
                        "reactivateValidator" => {
                            Some(TransactionKindDb::ReactivateValidator)
                        }
                        "deactivateValidator" => {
                            Some(TransactionKindDb::DeactivateValidator)
                        }
                        "unjailValidator" => {
                            Some(TransactionKindDb::UnjailValidator)
                        }
                        _ => None, // Skip invalid types
                    }
                })
                .collect()
        })
    }

    pub async fn get_addresses_history(
        &self,
        addresses: Vec<String>,
        page: u64,
        transaction_types: Option<String>,
    ) -> Result<(Vec<TransactionHistory>, u64, u64), TransactionError> {
        let parsed_types = Self::parse_transaction_types(transaction_types);

        let (txs, total_pages, total_items) = self
            .transaction_repo
            .find_addresses_history(addresses, page as i64, parsed_types)
            .await
            .map_err(TransactionError::Database)?;

        Ok((
            txs.into_iter()
                .map(|(h, t, bh)| TransactionHistory::from(h, t, bh))
                .collect(),
            total_pages as u64,
            total_items as u64,
        ))
    }
}
