use bigdecimal::BigDecimal;
use orm::bond::BondDb;
use orm::crawler_state::{BlockCrawlerStateDb, EpochCrawlerStateDb};
use orm::pos_rewards::PoSRewardDb;
use orm::unbond::UnbondDb;
use orm::validators::{ValidatorDb, ValidatorStateDb};
use serde::{Deserialize, Serialize};

use super::utils::{epoch_progress, time_between_epochs};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValidatorState {
    Consensus,
    BelowCapacity,
    BelowThreshold,
    Inactive,
    Jailed,
    Unknown,
}

impl From<ValidatorStateDb> for ValidatorState {
    fn from(value: ValidatorStateDb) -> Self {
        match value {
            ValidatorStateDb::Consensus => Self::Consensus,
            ValidatorStateDb::BelowCapacity => Self::BelowCapacity,
            ValidatorStateDb::BelowThreshold => Self::BelowThreshold,
            ValidatorStateDb::Inactive => Self::Inactive,
            ValidatorStateDb::Jailed => Self::Jailed,
            ValidatorStateDb::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub voting_power: String,
    pub max_commission: String,
    pub commission: String,
    pub state: ValidatorState,
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BondStatus {
    Active,
    Inactive,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bond {
    pub amount: String,
    pub validator: ValidatorWithId,
    pub status: BondStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MergedBond {
    pub amount: String,
    pub validator: ValidatorWithId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unbond {
    pub amount: String,
    pub validator: ValidatorWithId,
    pub withdraw_epoch: String,
    pub withdraw_time: String,
    pub current_time: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw {
    pub amount: String,
    pub validator: ValidatorWithId,
    pub withdraw_epoch: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub amount: String,
    pub validator: ValidatorWithId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalVotingPower {
    pub total_voting_power: String,
}

impl From<ValidatorDb> for Validator {
    fn from(value: ValidatorDb) -> Self {
        Self {
            address: value.namada_address,
            voting_power: value.voting_power.to_string(),
            max_commission: value.max_commission,
            commission: value.commission,
            state: value.state.into(),
            name: value.name,
            email: value.email,
            website: value.website,
            description: value.description,
            discord_handle: value.discord_handle,
            avatar: value.avatar,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorWithId {
    #[serde(flatten)]
    pub validator: Validator,
    pub validator_id: String,
}

impl ValidatorWithId {
    pub fn from(db_validator: ValidatorDb) -> Self {
        Self {
            validator_id: db_validator.id.to_string(),
            validator: Validator::from(db_validator),
        }
    }
}

impl From<(&BondDb, &EpochCrawlerStateDb)> for BondStatus {
    fn from((bond, status): (&BondDb, &EpochCrawlerStateDb)) -> Self {
        if bond.start <= status.last_processed_epoch {
            Self::Active
        } else {
            Self::Inactive
        }
    }
}

impl Bond {
    pub fn from(
        db_bond: BondDb,
        status: BondStatus,
        db_validator: ValidatorDb,
    ) -> Self {
        Self {
            amount: db_bond.raw_amount.to_string(),
            validator: ValidatorWithId::from(db_validator),
            status,
        }
    }
}

impl MergedBond {
    pub fn from(amount: BigDecimal, db_validator: ValidatorDb) -> Self {
        Self {
            amount: amount.to_string(),
            validator: ValidatorWithId::from(db_validator),
        }
    }
}

impl Unbond {
    pub fn from(
        // TODO: a lot of args
        raw_amount: BigDecimal,
        withdraw_epoch: i32,
        db_validator: ValidatorDb,
        chain_state: &BlockCrawlerStateDb,
        min_num_of_blocks: i32,
        min_duration: i32,
    ) -> Self {
        let epoch_progress =
            epoch_progress(chain_state.last_processed_block, min_num_of_blocks);

        let to_withdraw = time_between_epochs(
            min_num_of_blocks,
            epoch_progress,
            chain_state.last_processed_epoch,
            withdraw_epoch,
            min_duration,
        );

        let time_now = chain_state.timestamp.and_utc().timestamp();
        let withdraw_time = time_now + i64::from(to_withdraw);

        Self {
            amount: raw_amount.to_string(),
            validator: ValidatorWithId::from(db_validator),
            withdraw_epoch: withdraw_epoch.to_string(),
            withdraw_time: withdraw_time.to_string(),
            current_time: time_now.to_string(),
        }
    }
}

impl Withdraw {
    pub fn from(db_unbond: UnbondDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_unbond.raw_amount.to_string(),
            validator: ValidatorWithId::from(db_validator),
            withdraw_epoch: db_unbond.withdraw_epoch.to_string(),
        }
    }
}

impl Reward {
    pub fn from(db_reward: PoSRewardDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_reward.raw_amount.to_string(),
            validator: ValidatorWithId::from(db_validator),
        }
    }
}
