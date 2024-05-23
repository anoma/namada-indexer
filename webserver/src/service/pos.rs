use crate::appstate::AppState;
use crate::error::pos::PoSError;
use crate::repository::pos::{PosRepository, PosRepositoryTrait};
use crate::response::pos::{Bond, Reward, Unbond, ValidatorWithId, Withdraw};

#[derive(Clone)]
pub struct PosService {
    pos_repo: PosRepository,
}

impl PosService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            pos_repo: PosRepository::new(app_state),
        }
    }

    pub async fn get_all_validators(
        &self,
        page: u64,
    ) -> Result<(Vec<ValidatorWithId>, u64), PoSError> {
        let (db_validators, total_items) = self
            .pos_repo
            .find_all_validators(page as i64)
            .await
            .map_err(PoSError::Database)?;

        Ok((
            db_validators
                .into_iter()
                .map(ValidatorWithId::from)
                .collect(),
            total_items as u64,
        ))
    }

    pub async fn get_validators_by_delegator(
        &self,
        address: String,
        page: u64,
    ) -> Result<(Vec<ValidatorWithId>, u64), PoSError> {
        let (db_validators, total_items) = self
            .pos_repo
            .find_validators_by_delegator(address, page as i64)
            .await
            .map_err(PoSError::Database)?;

        Ok((
            db_validators
                .into_iter()
                .map(ValidatorWithId::from)
                .collect(),
            total_items as u64,
        ))
    }

    pub async fn get_bonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<Bond>, PoSError> {
        let db_bonds = self
            .pos_repo
            .find_bonds_by_address(address)
            .await
            .map_err(PoSError::Database)?;

        let mut bonds = vec![];
        for db_bond in db_bonds {
            let db_validator = self
                .pos_repo
                .find_validator_by_id(db_bond.validator_id)
                .await;
            if let Ok(Some(db_validator)) = db_validator {
                bonds.push(Bond::from(db_bond.clone(), db_validator));
            } else {
                tracing::error!(
                    "Couldn't find validator with id {} in bond query",
                    db_bond.validator_id
                );
            }
        }
        Ok(bonds)
    }

    pub async fn get_unbonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<Unbond>, PoSError> {
        let db_unbonds = self
            .pos_repo
            .find_unbonds_by_address(address)
            .await
            .map_err(PoSError::Database)?;

        let mut unbonds = vec![];
        for db_unbond in db_unbonds {
            let db_validator = self
                .pos_repo
                .find_validator_by_id(db_unbond.validator_id)
                .await;
            if let Ok(Some(db_validator)) = db_validator {
                unbonds.push(Unbond::from(db_unbond.clone(), db_validator));
            } else {
                tracing::error!(
                    "Couldn't find validator with id {} in bond query",
                    db_unbond.validator_id
                );
            }
        }
        Ok(unbonds)
    }

    pub async fn get_withdraws_by_address(
        &self,
        address: String,
        current_epoch: u64,
    ) -> Result<Vec<Withdraw>, PoSError> {
        let db_unbonds = self
            .pos_repo
            .find_withdraws_by_address(address, current_epoch as i32)
            .await
            .map_err(PoSError::Database)?;

        let mut unbonds = vec![];
        for db_unbond in db_unbonds {
            let db_validator = self
                .pos_repo
                .find_validator_by_id(db_unbond.validator_id)
                .await;
            if let Ok(Some(db_validator)) = db_validator {
                unbonds.push(Withdraw::from(db_unbond.clone(), db_validator));
            } else {
                tracing::error!(
                    "Couldn't find validator with id {} in bond query",
                    db_unbond.validator_id
                );
            }
        }
        Ok(unbonds)
    }

    pub async fn get_rewards_by_address(
        &self,
        address: String,
    ) -> Result<Vec<Reward>, PoSError> {
        let db_rewards = self
            .pos_repo
            .find_rewards_by_address(address)
            .await
            .map_err(PoSError::Database)?;

        let mut rewards = vec![];
        for db_reward in db_rewards {
            let db_validator = self
                .pos_repo
                .find_validator_by_id(db_reward.validator_id)
                .await;
            if let Ok(Some(db_validator)) = db_validator {
                rewards.push(Reward::from(db_reward.clone(), db_validator));
            } else {
                tracing::error!(
                    "Couldn't find validator with id {} in bond query",
                    db_reward.validator_id
                );
            }
        }
        Ok(rewards)
    }

    pub async fn get_total_voting_power(&self) -> Result<u64, PoSError> {
        let total_voting_power_db = self
            .pos_repo
            .get_total_voting_power()
            .await
            .map_err(PoSError::Database)?;

        Ok(total_voting_power_db.unwrap_or_default() as u64)
    }
}
