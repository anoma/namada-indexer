use crate::appstate::AppState;
use crate::service::pos::PosService;

#[derive(Clone)]
pub struct CommonState {
    pub pos_service: PosService,
}

impl CommonState {
    pub fn new(data: AppState) -> Self {
        Self {
            pos_service: PosService::new(data.clone()),
        }
    }
}
