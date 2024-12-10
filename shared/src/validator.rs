use std::collections::HashSet;

use fake::faker::company::en::{CatchPhrase, CompanyName};
use fake::faker::internet::en::{DomainSuffix, SafeEmail, Username};
use fake::Fake;
use namada_proof_of_stake::types::ValidatorState as NamadaValidatorState;
use rand::distributions::{Distribution, Standard};

use crate::block::Epoch;
use crate::id::Id;

pub type VotingPower = String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ValidatorState {
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

impl From<NamadaValidatorState> for ValidatorState {
    fn from(value: NamadaValidatorState) -> Self {
        match value {
            NamadaValidatorState::Consensus => ValidatorState::Consensus,
            NamadaValidatorState::BelowCapacity => {
                ValidatorState::BelowCapacity
            }
            NamadaValidatorState::BelowThreshold => {
                ValidatorState::BelowThreshold
            }
            NamadaValidatorState::Inactive => ValidatorState::Inactive,
            NamadaValidatorState::Jailed => ValidatorState::Jailed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatorSet {
    pub validators: HashSet<Validator>,
    pub epoch: Epoch,
}

impl ValidatorSet {
    pub fn union(&self, validator_set: &ValidatorSet) -> Self {
        ValidatorSet {
            validators: self
                .validators
                .union(&validator_set.validators)
                .cloned()
                .collect::<HashSet<Validator>>(),
            epoch: self.epoch,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Validator {
    pub address: Id,
    pub voting_power: VotingPower,
    pub max_commission: String,
    pub commission: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handler: Option<String>,
    pub avatar: Option<String>,
    pub state: ValidatorState,
}

#[derive(Debug, Clone)]
pub struct ValidatorMetadataChange {
    pub address: Id,
    pub commission: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handler: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidatorStateChange {
    pub address: Id,
    pub state: ValidatorState,
}

impl Validator {
    pub fn fake() -> Self {
        let address =
            namada_core::address::gen_established_address("namada-indexer");
        let voting_power = (1000..10000000).fake::<u64>().to_string();
        let max_commission =
            ((0..100).fake::<u64>() as f64 / 100_f64).to_string();
        let commission = ((0..100).fake::<u64>() as f64 / 100_f64).to_string();
        let email = Some(SafeEmail().fake());
        let description: Option<String> = CatchPhrase().fake();
        let name = Some(CompanyName().fake::<String>());
        let website: Option<String> = Some(format!(
            "{}.{}",
            name.clone().unwrap(),
            DomainSuffix().fake::<String>()
        ));
        let discord_handler: Option<String> = Username().fake();

        Self {
            address: Id::Account(address.to_string()),
            voting_power,
            max_commission,
            commission,
            name,
            email,
            description,
            website,
            discord_handler,
            avatar: Some("https://picsum.photos/200/300".to_string()),
            state: rand::random(),
        }
    }
}

impl Distribution<ValidatorState> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> ValidatorState {
        match rng.gen_range(0..=5) {
            0 => ValidatorState::Consensus,
            1 => ValidatorState::Inactive,
            2 => ValidatorState::Jailed,
            3 => ValidatorState::BelowCapacity,
            _ => ValidatorState::BelowThreshold,
        }
    }
}
