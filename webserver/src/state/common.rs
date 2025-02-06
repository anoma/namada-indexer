use namada_sdk::tendermint_rpc::HttpClient;

use crate::appstate::AppState;
use crate::config::AppConfig;
use crate::service::balance::BalanceService;
use crate::service::block::BlockService;
use crate::service::chain::ChainService;
use crate::service::crawler_state::CrawlerStateService;
use crate::service::gas::GasService;
use crate::service::governance::GovernanceService;
use crate::service::ibc::IbcService;
use crate::service::pgf::PgfService;
use crate::service::pos::PosService;
use crate::service::revealed_pk::RevealedPkService;
use crate::service::transaction::TransactionService;

#[derive(Clone)]
pub struct CommonState {
    pub pos_service: PosService,
    pub block_service: BlockService,
    pub gov_service: GovernanceService,
    pub balance_service: BalanceService,
    pub chain_service: ChainService,
    pub revealed_pk_service: RevealedPkService,
    pub gas_service: GasService,
    pub transaction_service: TransactionService,
    pub pgf_service: PgfService,
    pub crawler_state_service: CrawlerStateService,
    pub ibc_service: IbcService,
    pub client: HttpClient,
    pub config: AppConfig,
}

impl CommonState {
    pub fn new(client: HttpClient, config: AppConfig, data: AppState) -> Self {
        Self {
            block_service: BlockService::new(data.clone()),
            pos_service: PosService::new(data.clone()),
            gov_service: GovernanceService::new(data.clone()),
            balance_service: BalanceService::new(data.clone()),
            chain_service: ChainService::new(data.clone()),
            revealed_pk_service: RevealedPkService::new(data.clone()),
            gas_service: GasService::new(data.clone()),
            pgf_service: PgfService::new(data.clone()),
            transaction_service: TransactionService::new(data.clone()),
            crawler_state_service: CrawlerStateService::new(data.clone()),
            ibc_service: IbcService::new(data.clone()),
            client,
            config,
        }
    }
}
