use core::fmt;
use std::fmt::Display;

use shared::log_config::LogConfig;

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum CargoEnv {
    Development,
    Production,
}

impl Display for CargoEnv {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env)]
    pub tendermint_url: String,

    #[clap(long, env, default_value_t = 1)]
    pub from_block_height: u32,

    #[clap(
        long,
        help = "Crawl from given height and do not update crawler_state"
    )]
    pub backfill_from: Option<u32>,

    #[clap(long, env)]
    pub database_url: String,

    #[clap(flatten)]
    pub log: LogConfig,
}
