#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum CargoEnv {
    Development,
    Production,
}

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(long, env, default_value = "5000")]
    pub port: u16,

    #[clap(long, env)]
    pub cache_url: String,

    #[clap(long, env)]
    pub database_url: String,

    #[clap(long, env)]
    pub rps: Option<u64>,

    #[clap(long, env)]
    pub tendermint_url: String,
}
