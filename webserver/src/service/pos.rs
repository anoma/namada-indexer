use shared::id::Id;
use shared::validator::Validator;

use crate::appstate::AppState;
use crate::repository::pos::{PosRepository, PosRepositoryTrait};

#[derive(Clone)]
pub struct PosService {
    pos_repo: PosRepository,
}

impl PosService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            pos_repo: PosRepository::new(app_state),
        }
    }

    pub async fn get_all_validators(&self) -> Vec<Validator> {
        let validators = self.pos_repo.find_all_validators().await;
        validators
            .unwrap_or_default()
            .into_iter()
            .map(|v| Validator {
                address: Id::Account(v.namada_address),
                voting_power: v.voting_power.to_string(),
                max_commission: v.max_commission,
                commission: v.commission,
                email: v.email,
                description: v.description,
                website: v.website,
                discord_handler: v.discord_handle,
                avatar: v.avatar,
            })
            .collect()
    }
}
