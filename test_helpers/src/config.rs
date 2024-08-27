#[derive(clap::Parser)]
pub struct TestConfig {
    #[clap(long, env)]
    pub database_url_test: String,
}
