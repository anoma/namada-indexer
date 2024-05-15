use crate::appstate::AppState;
use crate::error::pos::PoSError;
use crate::repository::pos::{PosRepository, PosRepositoryTrait};
use crate::response::pos::Validator;

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

    pub async fn get_all_validators(
        &self,
        page: u64,
    ) -> Result<(Vec<Validator>, u64), PoSError> {
        let (db_validators, total_items) = self
            .pos_repo
            .find_all_validators(page as i64)
            .await
            .map_err(PoSError::Database)?;

        Ok((
            db_validators.into_iter().map(Validator::from).collect(),
            total_items as u64,
        ))
    }
}
