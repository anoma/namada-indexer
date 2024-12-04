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
    pub database_url: String,

    #[clap(long, env, default_value_t = 5)]
    pub total_validators: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_proposals: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_votes: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_rewards: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_bonds: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_unbonds: u64,

    #[clap(long, env, default_value_t = 10)]
    pub total_balances: u64,

    #[clap(flatten)]
    pub log: LogConfig,
}
