use bigdecimal::BigDecimal;
use orm::ibc::IbcAckStatusDb;

use crate::appstate::AppState;
use crate::error::ibc::IbcError;
use crate::repository::ibc::{IbcRepository, IbcRepositoryTrait};
use crate::response::ibc::{IbcAck, IbcAckStatus, IbcRateLimit};

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
                    timeout: Some(ack.timeout),
                },
                None => IbcAck {
                    status: IbcAckStatus::Unknown,
                    timeout: None,
                },
            })
    }

    pub async fn get_throughput_limits(
        &self,
        matching_token_address: Option<String>,
        matching_rate_limit: Option<BigDecimal>,
    ) -> Result<Vec<IbcRateLimit>, IbcError> {
        const _: () = {
            // NB: Statically assert that the cast we will
            // perform below is safe.
            if std::mem::size_of::<(String, String)>()
                != std::mem::size_of::<IbcRateLimit>()
            {
                panic!("IbcRateLimit size is invalid");
            }

            if std::mem::align_of::<(String, String)>()
                != std::mem::align_of::<IbcRateLimit>()
            {
                panic!("IbcRateLimit alignment is invalid");
            }
        };

        self.ibc_repo
            .get_throughput_limits(matching_token_address, matching_rate_limit)
            .await
            .map_err(IbcError::Database)
            .map(|limits| unsafe {
                // NB: Transmute this value. It's faster than destructing
                // the vec and creating a new one, just to convert between
                // types.

                // SAFETY: We have asserted above that `IbcRateLimit` is
                // compatible with the type `(String, BigDecimal)`.
                std::mem::transmute(limits)
            })
    }
}
