use crate::appstate::AppState;
use crate::error::transaction::TransactionError;
use crate::repository::tranasaction::{
    TransactionRepository, TransactionRepositoryTrait,
};
use crate::response::transaction::{InnerTransaction, WrapperTransaction};

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

        Ok(wrapper_tx.map(WrapperTransaction::from))
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
}
