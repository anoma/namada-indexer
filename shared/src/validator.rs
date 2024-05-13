use fake::faker::company::en::{CompanyName, CatchPhase};
use fake::faker::internet::en::{DomainSuffix, SafeEmail, Username};
use fake::Fake;


use crate::block::Epoch;
use crate::id::Id;

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
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handler: Option<String>,
    pub avatar: Option<String>,
}

impl Validator {
    pub fn fake() -> Self {
        let address = namada_core::address::gen_established_address("123");
        let voting_power = (1000..10000000).fake::<u64>().to_string();
        let max_commission = ((0..100).fake::<u64>() as f64 / 100 as f64).to_string();
        let commission = ((0..100).fake::<u64>() as f64 / 100 as f64).to_string();
        let email: String = SafeEmail().fake();
        let description: Option<String> = CatchPhase().fake();
        let website: Option<String> = Some(format!("{}.{}", CompanyName().fake::<String>(), DomainSuffix().fake::<String>()));
        let discord_handler: Option<String> = Username().fake();

        Self {
            address: Id::Account(address.to_string()),
            voting_power,
            max_commission,
            commission,
            email,
            description,
            website,
            discord_handler,
            avatar: Some("https://picsum.photos/200/300".to_string()),
        }
    }
}