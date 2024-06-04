use std::str::FromStr;

use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use shared::validator::{Validator, ValidatorState};

use crate::schema::validators;

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ValidatorState"]
pub enum ValidatorStateDb {
    Consensus,
    BelowCapacity,
    BelowThreshold,
    Inactive,
    Jailed,
    Unknown,
}

impl From<ValidatorState> for ValidatorStateDb {
    fn from(value: ValidatorState) -> Self {
        match value {
            ValidatorState::Consensus => Self::Consensus,
            ValidatorState::BelowCapacity => Self::BelowCapacity,
            ValidatorState::BelowThreshold => Self::BelowThreshold,
            ValidatorState::Inactive => Self::Inactive,
            ValidatorState::Jailed => Self::Jailed,
            ValidatorState::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorDb {
    pub id: i32,
    pub namada_address: String,
    pub voting_power: i32,
    pub max_commission: String,
    pub commission: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
    pub state: ValidatorStateDb,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorInsertDb {
    pub namada_address: String,
    pub voting_power: i32,
    pub max_commission: String,
    pub commission: String,
}

#[derive(Serialize, AsChangeset, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorUpdateMetadataDb {
    pub commission: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
}

impl ValidatorInsertDb {
    pub fn from_validator(validator: Validator) -> Self {
        Self {
            namada_address: validator.address.to_string(),
            voting_power: f32::from_str(&validator.voting_power).unwrap()
                as i32,
            max_commission: validator.max_commission.clone(),
            commission: validator.commission.clone(),
        }
    }
}
