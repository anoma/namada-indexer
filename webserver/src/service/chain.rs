use crate::{appstate::AppState, repository::chain::{ChainRepository, ChainRepositoryTrait}};

#[derive(Clone)]
pub struct ChainService {
    chain_repo: ChainRepository,
}

impl ChainService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            chain_repo: ChainRepository::new(app_state),
        }
    }

    // return Result with ChainError
    pub async fn find_latest_height(&self) -> u64 {
        self.chain_repo
            .find_latest_height()
            .await
            .unwrap()
            .unwrap_or_default() as u64
    }
}
