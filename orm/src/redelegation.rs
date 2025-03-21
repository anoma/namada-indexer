use diesel::associations::Associations;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use shared::pos::Redelegation;

use crate::schema::redelegation;
use crate::validators::ValidatorDb;

#[derive(Insertable, Clone, Queryable, Selectable)]
#[diesel(table_name = redelegation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RedelegationInsertDb {
    pub delegator: String,
    pub validator_id: i32,
    pub end_epoch: i32,
}

#[derive(Identifiable, Clone, Queryable, Selectable, Associations)]
#[diesel(table_name = redelegation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(ValidatorDb, foreign_key = validator_id))]
pub struct RedelegationDb {
    pub id: i32,
    pub delegator: String,
    pub validator_id: i32,
    pub end_epoch: i32,
}

impl RedelegationInsertDb {
    pub fn from_redelegation(
        redelegation: Redelegation,
        validator_id: i32,
    ) -> Self {
        Self {
            delegator: redelegation.delegator.to_string(),
            validator_id,
            end_epoch: redelegation.end_epoch as i32,
        }
    }
}
