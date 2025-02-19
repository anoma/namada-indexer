use bigdecimal::BigDecimal;
use orm::ibc::IbcAckStatusDb;

use crate::appstate::AppState;
use crate::error::ibc::IbcError;
use crate::repository::ibc::{IbcRepository, IbcRepositoryTrait};
use crate::response::ibc::{
    IbcAck, IbcAckStatus, IbcRateLimit, IbcTokenFlow, IbcTokenThroughput,
};

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
        self.ibc_repo
            .get_throughput_limits(matching_token_address, matching_rate_limit)
            .await
            .map_err(IbcError::Database)
            .map(|limits| unsafe {
                // NB: Transmute this value. It's faster than destructing
                // the vec and creating a new one, just to convert between
                // types.

                const _: () =
                    assert_conversion_safety::<(String, String), IbcRateLimit>(
                    );

                // SAFETY: We have asserted the safety of the conversion above
                std::mem::transmute(limits)
            })
    }

    pub async fn get_token_flows(
        &self,
        matching_token_address: Option<String>,
    ) -> Result<Vec<IbcTokenFlow>, IbcError> {
        self.ibc_repo
            .get_token_flows(matching_token_address)
            .await
            .map_err(IbcError::Database)
            .map(|flows| unsafe {
                // NB: Transmute this value. It's faster than destructing
                // the vec and creating a new one, just to convert between
                // types.

                const _: () = assert_conversion_safety::<
                    (String, String, String),
                    IbcTokenFlow,
                >();

                // SAFETY: We have asserted the safety of the conversion above
                std::mem::transmute(flows)
            })
    }

    pub async fn get_token_throughput(
        &self,
        token: String,
    ) -> Result<IbcTokenThroughput, IbcError> {
        self.ibc_repo
            .get_token_throughput(token)
            .await
            .map_err(IbcError::Database)
            .map(|throughput| unsafe {
                // NB: Transmute this value. It's faster than destructing
                // the vec and creating a new one, just to convert between
                // types.

                const _: () = assert_conversion_safety::<
                    (String, String),
                    IbcTokenThroughput,
                >();

                // SAFETY: We have asserted the safety of the conversion above
                std::mem::transmute(throughput)
            })
    }
}

#[allow(dead_code)]
const fn assert_conversion_safety<From, To>() {
    if std::mem::size_of::<From>() != std::mem::size_of::<To>() {
        panic!("size is invalid");
    }

    if std::mem::align_of::<From>() != std::mem::align_of::<To>() {
        panic!("alignment is invalid");
    }
}
