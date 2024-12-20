use std::collections::HashMap;

use crate::appstate::AppState;
use crate::error::gas::GasError;
use crate::repository::gas::{GasRepository, GasRepositoryTrait};
use crate::response::gas::{Gas, GasPrice};
use crate::response::transaction::TransactionKind;

#[derive(Clone)]
pub struct GasService {
    gas_repo: GasRepository,
}

impl GasService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            gas_repo: GasRepository::new(app_state),
        }
    }

    pub async fn get_gas(&self) -> Vec<Gas> {
        self.gas_repo
            .get_gas()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Gas::from)
            .collect()
    }

    pub async fn get_gas_price_by_token(
        &self,
        token: String,
    ) -> Result<Vec<GasPrice>, GasError> {
        self.gas_repo
            .find_gas_price_by_token(token)
            .await
            .map_err(GasError::Database)
            .map(|r| r.iter().cloned().map(GasPrice::from).collect())
    }

    pub async fn get_all_gas_prices(&self) -> Result<Vec<GasPrice>, GasError> {
        self.gas_repo
            .find_all_gas_prices()
            .await
            .map_err(GasError::Database)
            .map(|r| r.iter().cloned().map(GasPrice::from).collect())
    }

    pub async fn estimate_gas(
        &self,
        bond: u64,
        claim_rewards: u64,
        unbond: u64,
        transparent_transfer: u64,
        shielded_transfer: u64,
        shielding_transfer: u64,
        unshielding_transfer: u64,
        vote: u64,
        ibc: u64,
        withdraw: u64,
        reveal_pk: u64
    ) -> u64 {
        let gas = self
            .gas_repo
            .get_gas()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(Gas::from)
            .fold(HashMap::new(), |mut acc, gas| {
                acc.insert(gas.tx_kind, gas.gas_limit);
                acc
            });

        let mut estimate = 0;
        estimate += bond * gas.get(&TransactionKind::Bond).unwrap();
        estimate +=
            claim_rewards * gas.get(&TransactionKind::ClaimRewards).unwrap();
        estimate += unbond * gas.get(&TransactionKind::Unbond).unwrap();
        estimate += transparent_transfer
            * gas.get(&TransactionKind::TransparentTransfer).unwrap();
        estimate += shielded_transfer
            * gas.get(&TransactionKind::ShieldedTransfer).unwrap();
        estimate += shielding_transfer
            * gas.get(&TransactionKind::ShieldingTransfer).unwrap();
        estimate += unshielding_transfer
            * gas.get(&TransactionKind::UnshieldingTransfer).unwrap();
        estimate += vote * gas.get(&TransactionKind::VoteProposal).unwrap();
        estimate += ibc * gas.get(&TransactionKind::IbcMsgTransfer).unwrap();
        estimate += withdraw * gas.get(&TransactionKind::Withdraw).unwrap();
        estimate += reveal_pk * gas.get(&TransactionKind::RevealPk).unwrap();

        estimate
    }
}
