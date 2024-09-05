use std::sync::Arc;

use namada_sdk::tendermint_rpc::HttpClient;
use shared::event_store::PosEvents;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::service::balance::BalanceService;
use crate::service::chain::ChainService;
use crate::service::crawler_state::CrawlerStateService;
use crate::service::gas::GasService;
use crate::service::governance::GovernanceService;
use crate::service::pos::PosService;
use crate::service::revealed_pk::RevealedPkService;
use crate::service::transaction::TransactionService;

#[derive(Clone)]
pub struct CommonState {
    pub pos_service: PosService,
    pub gov_service: GovernanceService,
    pub balance_service: BalanceService,
    pub chain_service: ChainService,
    pub revealed_pk_service: RevealedPkService,
    pub gas_service: GasService,
    pub transaction_service: TransactionService,
    pub crawler_state_service: CrawlerStateService,
    pub client: HttpClient,
    pub config: AppConfig,
    pub events_rx: Arc<Mutex<Receiver<PosEvents>>>,
}

impl CommonState {
    pub fn new(
        client: HttpClient,
        config: AppConfig,
        data: AppState,
        events_rx: Arc<Mutex<Receiver<PosEvents>>>,
    ) -> Self {
        Self {
            pos_service: PosService::new(data.clone()),
            gov_service: GovernanceService::new(data.clone()),
            balance_service: BalanceService::new(data.clone()),
            chain_service: ChainService::new(data.clone()),
            revealed_pk_service: RevealedPkService::new(data.clone()),
            gas_service: GasService::new(data.clone()),
            transaction_service: TransactionService::new(data.clone()),
            crawler_state_service: CrawlerStateService::new(data.clone()),
            client,
            config,
            events_rx,
        }
    }
}
