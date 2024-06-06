use orm::validators::ValidatorStateDb;

use super::utils::raw_amount_to_nam;
use crate::appstate::AppState;
use crate::dto::pos::{MyValidatorKindDto, ValidatorStateDto};
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
        states: Vec<ValidatorStateDto>,
    ) -> Result<(Vec<ValidatorWithId>, u64), PoSError> {
        let validator_states = states
            .into_iter()
            .map(|state| match state {
                ValidatorStateDto::Consensus => ValidatorStateDb::Consensus,
                ValidatorStateDto::BelowCapacity => {
                    ValidatorStateDb::BelowCapacity
                }
                ValidatorStateDto::BelowThreshold => {
                    ValidatorStateDb::BelowThreshold
                }
                ValidatorStateDto::Inactive => ValidatorStateDb::Inactive,
                ValidatorStateDto::Jailed => ValidatorStateDb::Jailed,
                ValidatorStateDto::Unknown => ValidatorStateDb::Unknown,
            })
            .collect();
        let (db_validators, total_items) = self
            .pos_repo
            .find_all_validators(page as i64, validator_states)
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

    pub async fn get_my_validators(
        &self,
        page: u64,
        addresses: Vec<String>,
        kind: MyValidatorKindDto,
    ) -> Result<(Vec<ValidatorWithId>, u64), PoSError> {
        let (db_validators, total_items) = self
            .pos_repo
            .find_validators_by_addresses(page as i64, addresses, kind)
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
        // TODO: could optimize and make a single query
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

        let denominated_bonds: Vec<Bond> = bonds
            .iter()
            .cloned()
            .map(|bond| Bond {
                amount: raw_amount_to_nam(bond.amount),
                ..bond
            })
            .collect();
        Ok(denominated_bonds)
    }

    pub async fn get_unbonds_by_address(
        &self,
        address: String,
    ) -> Result<Vec<Unbond>, PoSError> {
        // TODO: could optimize and make a single query
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
        let denominated_unbonds: Vec<Unbond> = unbonds
            .iter()
            .cloned()
            .map(|unbond| Unbond {
                amount: raw_amount_to_nam(unbond.amount),
                ..unbond
            })
            .collect();
        Ok(denominated_unbonds)
    }

    pub async fn get_withdraws_by_address(
        &self,
        address: String,
        current_epoch: u64,
    ) -> Result<Vec<Withdraw>, PoSError> {
        // TODO: could optimize and make a single query
        let db_unbonds = self
            .pos_repo
            .find_withdraws_by_address(address, current_epoch as i32)
            .await
            .map_err(PoSError::Database)?;

        let mut withdraws = vec![];
        for db_unbond in db_unbonds {
            let db_validator = self
                .pos_repo
                .find_validator_by_id(db_unbond.validator_id)
                .await;
            if let Ok(Some(db_validator)) = db_validator {
                withdraws.push(Withdraw::from(db_unbond.clone(), db_validator));
            } else {
                tracing::error!(
                    "Couldn't find validator with id {} in bond query",
                    db_unbond.validator_id
                );
            }
        }
        let denominated_withdraw: Vec<Withdraw> = withdraws
            .iter()
            .cloned()
            .map(|withdraw| Withdraw {
                amount: raw_amount_to_nam(withdraw.amount),
                ..withdraw
            })
            .collect();
        Ok(denominated_withdraw)
    }

    pub async fn get_rewards_by_address(
        &self,
        address: String,
    ) -> Result<Vec<Reward>, PoSError> {
        // TODO: could optimize and make a single query
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
        let denominated_rewards: Vec<Reward> = rewards
            .iter()
            .cloned()
            .map(|reward| Reward {
                amount: raw_amount_to_nam(reward.amount),
                ..reward
            })
            .collect();
        Ok(denominated_rewards)
    }

    // TODO: maybe remove object(struct) instead
    pub async fn get_total_voting_power(&self) -> Result<u64, PoSError> {
        let total_voting_power_db = self
            .pos_repo
            .get_total_voting_power()
            .await
            .map_err(PoSError::Database)?;

        Ok(total_voting_power_db.unwrap_or_default() as u64)
    }
}
