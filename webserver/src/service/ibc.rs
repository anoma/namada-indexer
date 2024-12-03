use orm::ibc::IbcAckStatusDb;

use crate::appstate::AppState;
use crate::error::ibc::IbcError;
use crate::repository::ibc::{IbcRepository, IbcRepositoryTrait};
use crate::response::ibc::{IbcAck, IbcAckStatus};

#[derive(Clone)]
pub struct IbcService {
    pub ibc_repo: IbcRepository,
}

impl IbcService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            ibc_repo: IbcRepository::new(app_state),
        }
    }

    pub async fn get_ack_by_tx_id(
        &self,
        tx_id: String,
    ) -> Result<IbcAck, IbcError> {
        self.ibc_repo
            .find_ibc_ack(tx_id)
            .await
            .map_err(IbcError::Database)
            .map(|ack| match ack {
                Some(ack) => IbcAck {
                    status: match ack.status {
                        IbcAckStatusDb::Unknown => IbcAckStatus::Unknown,
                        IbcAckStatusDb::Timeout => IbcAckStatus::Timeout,
                        IbcAckStatusDb::Fail => IbcAckStatus::Fail,
                        IbcAckStatusDb::Success => IbcAckStatus::Success,
                    },
                },
                None => IbcAck {
                    status: IbcAckStatus::Unknown,
                },
            })
    }
}
