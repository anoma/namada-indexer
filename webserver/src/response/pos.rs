use orm::validators::ValidatorDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Validator {
    pub address: String,
    pub voting_power: String,
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub discord_handle: Option<String>,
    pub avatar: Option<String>,
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
