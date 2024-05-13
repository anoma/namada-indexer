use core::fmt;
use std::fmt::Display;
use std::path::PathBuf;

use clap_verbosity_flag::{InfoLevel, Verbosity};

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

// TODO: remove unused fields
#[derive(clap::Parser)]
pub struct AppConfig {
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
