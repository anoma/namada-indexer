use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use rand::Rng;
use seeder::config::AppConfig;
use seeder::state::AppState;
use shared::error::AsDbError;
use shared::proposal::GovernanceProposal;
use shared::validator::Validator;
use shared::vote::GovernanceVote;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    let log_level = match config.verbosity.log_level_filter() {
        LevelFilter::Off => None,
        LevelFilter::Error => Some(Level::ERROR),
        LevelFilter::Warn => Some(Level::WARN),
        LevelFilter::Info => Some(Level::INFO),
        LevelFilter::Debug => Some(Level::DEBUG),
        LevelFilter::Trace => Some(Level::TRACE),
    };

    if let Some(log_level) = log_level {
        let subscriber =
            FmtSubscriber::builder().with_max_level(log_level).finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    tracing::info!("version: {}", env!("VERGEN_GIT_SHA").to_string());

    let validators = (0..config.total_validators)
        .map(|_| Validator::fake())
        .collect::<Vec<Validator>>();

    println!("{:#?}", validators);

    let governance_proposals = (0..config.total_proposals)
        .map(|id| GovernanceProposal::fake(id))
        .collect::<Vec<GovernanceProposal>>();

    println!("{:#?}", governance_proposals);

    let governance_votes = (0..config.total_votes).map(|_| {
        let proposal_id = rand::thread_rng().gen_range(0..config.total_proposals);
        GovernanceVote::fake(proposal_id)
    }).collect::<Vec<GovernanceVote>>();

    println!("{:#?}", governance_votes);

    let _app_state = AppState::new(config.database_url).into_db_error()?;

    Ok(())
}
