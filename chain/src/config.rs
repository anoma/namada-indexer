use clap_verbosity_flag::{InfoLevel, Verbosity};
use core::fmt;
use std::{fmt::Display, path::PathBuf};

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

    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub chain_id: String,

    #[clap(long, env)]
    pub checksums_filepath: PathBuf,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}
