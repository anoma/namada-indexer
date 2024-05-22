use orm::bond::BondDb;
use orm::pos_rewards::PoSRewardDb;
use orm::unbond::UnbondDb;
use orm::validators::ValidatorDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub voting_power: String,
    pub max_commission: String,
    pub commission: String,
    pub email: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Bond {
    pub amount: String,
    pub validator: ValidatorWithId,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Unbond {
    pub amount: String,
    pub validator: ValidatorWithId,
    pub withdraw_epoch: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw {
    pub amount: String,
    pub validator: ValidatorWithId,
    pub withdraw_epoch: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub amount: String,
    pub validator: ValidatorWithId,
}

impl From<ValidatorDb> for Validator {
    fn from(value: ValidatorDb) -> Self {
        Self {
            address: value.namada_address,
            voting_power: value.voting_power.to_string(),
            max_commission: value.max_commission,
            commission: value.commission,
            email: value.email,
            website: value.website,
            description: value.description,
            discord_handle: value.discord_handle,
            avatar: value.avatar,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorWithId {
    #[serde(flatten)]
    pub validator: Validator,
    pub validator_id: u64,
}

impl ValidatorWithId {
    pub fn from(db_validator: ValidatorDb) -> Self {
        Self {
            validator_id: db_validator.id as u64,
            validator: Validator::from(db_validator),
        }
    }
}

impl Bond {
    pub fn from(db_bond: BondDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_bond.raw_amount,
            validator: ValidatorWithId::from(db_validator),
        }
    }
}

impl Unbond {
    pub fn from(db_unbond: UnbondDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_unbond.raw_amount,
            validator: ValidatorWithId::from(db_validator),
            withdraw_epoch: db_unbond.withdraw_epoch as u64,
        }
    }
}

impl Withdraw {
    pub fn from(db_unbond: UnbondDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_unbond.raw_amount,
            validator: ValidatorWithId::from(db_validator),
            withdraw_epoch: db_unbond.withdraw_epoch as u64,
        }
    }
}

impl Reward {
    pub fn from(db_reward: PoSRewardDb, db_validator: ValidatorDb) -> Self {
        Self {
            amount: db_reward.raw_amount,
            validator: ValidatorWithId::from(db_validator),
        }
    }
}
