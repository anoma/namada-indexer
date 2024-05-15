use crate::appstate::AppState;
use crate::service::governance::GovernanceService;
use crate::service::pos::PosService;

#[derive(Clone)]
pub struct CommonState {
    pub pos_service: PosService,
    pub gov_service: GovernanceService,
}

impl CommonState {
    pub fn new(data: AppState) -> Self {
        Self {
            pos_service: PosService::new(data.clone()),
            gov_service: GovernanceService::new(data.clone()),
        }
    }
}
