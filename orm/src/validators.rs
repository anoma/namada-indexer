use std::str::FromStr;

use diesel::expression::expression_types::NotSelectable;
use diesel::sql_types::Nullable;
use diesel::{
    AsChangeset, BoxableExpression, ExpressionMethods, Insertable, Queryable,
    Selectable,
};
use serde::{Deserialize, Serialize};
use shared::validator::{Validator, ValidatorState};

use crate::helpers::OrderByDb;
use crate::schema::validators;
use crate::{asc_desc, rev_asc_desc};

#[derive(Debug)]
pub enum ValidatorSortByDb {
    VotingPower,
    Commission,
    Rank,
}

#[derive(Debug, Clone, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ValidatorState"]
pub enum ValidatorStateDb {
    Consensus,
    BelowCapacity,
    BelowThreshold,
    Inactive,
    Jailed,
    Deactivating,
    Reactivating,
    Unjailing,
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
            ValidatorState::Deactivating => Self::Deactivating,
            ValidatorState::Reactivating => Self::Reactivating,
            ValidatorState::Unjailing => Self::Unjailing,
            ValidatorState::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Clone, Debug)]
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
    pub state: ValidatorStateDb,
}

#[derive(Serialize, Insertable, Clone)]
#[diesel(table_name = validators)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ValidatorWithMetaInsertDb {
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
pub struct ValidatorStateChangeDb {
    pub namada_address: String,
    pub state: ValidatorStateDb,
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
            state: validator.state.into(),
        }
    }
}

impl ValidatorWithMetaInsertDb {
    pub fn from_validator(validator: Validator) -> Self {
        Self {
            namada_address: validator.address.to_string(),
            voting_power: f32::from_str(&validator.voting_power).unwrap()
                as i32,
            max_commission: validator.max_commission.clone(),
            commission: validator.commission.clone(),
            name: validator.name,
            email: validator.email,
            website: validator.website,
            description: validator.description,
            discord_handle: validator.discord_handler,
            avatar: validator.avatar,
            state: validator.state.into(),
        }
    }
}

pub fn validator_sort_by(
    validator_sort_by: ValidatorSortByDb,
    order: OrderByDb,
) -> Box<
    dyn BoxableExpression<
            validators::table,
            diesel::pg::Pg,
            SqlType = NotSelectable,
        >,
> {
    match validator_sort_by {
        ValidatorSortByDb::VotingPower => {
            asc_desc!(order, validators::columns::voting_power)
        }
        ValidatorSortByDb::Commission => {
            asc_desc!(order, validators::columns::commission)
        }
        ValidatorSortByDb::Rank => {
            rev_asc_desc!(order, validators::columns::voting_power)
        }
    }
}
