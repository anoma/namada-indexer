use crate::{block::Epoch, id::Id};

pub type VotingPower = String;

#[derive(Debug, Clone)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub epoch: Epoch,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub address: Id,
    pub voting_power: VotingPower,
    pub tm_address: Id,
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handler: Option<String>,
    pub avatar: Option<String>,
}
