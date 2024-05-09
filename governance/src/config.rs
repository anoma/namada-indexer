use clap_verbosity_flag::{InfoLevel, Verbosity};
use core::fmt;
use std::fmt::Display;

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
    #[clap(long, env, value_enum)]
    pub cargo_env: CargoEnv,

    #[clap(long, env)]
    pub tendermint_url: String,

    #[clap(long, env, default_value_t = 900)]
    pub sleep_for: u64,

    #[clap(long, env)]
    pub database_url: String,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}
