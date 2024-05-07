use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ValidatorResponse {
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
