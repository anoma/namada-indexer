use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::RunQueryDsl;
use orm::governance_proposal::GovernanceProposalInsertDb;
use orm::governance_votes::GovernanceProposalVoteInsertDb;
use orm::pos_rewards::PosRewardInsertDb;
use orm::schema::{
    governance_proposals, governance_votes, pos_rewards, validators,
};
use orm::validators::ValidatorInsertDb;
use rand::Rng;
use seeder::config::AppConfig;
use seeder::state::AppState;
use shared::error::{AsDbError, ContextDbInteractError, MainError};
use shared::proposal::GovernanceProposal;
use shared::rewards::Reward;
use shared::validator::Validator;
use shared::vote::GovernanceVote;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<(), MainError> {
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

    let governance_proposals = (0..config.total_proposals)
        .map(|id| GovernanceProposal::fake(id))
        .collect::<Vec<GovernanceProposal>>();

    let governance_votes = (0..config.total_votes)
        .map(|_| {
            let proposal_id =
                rand::thread_rng().gen_range(0..config.total_proposals);
            GovernanceVote::fake(proposal_id)
        })
        .collect::<Vec<GovernanceVote>>();

    let rewards = (0..config.total_rewards)
        .map(|_| Reward::fake())
        .collect::<Vec<Reward>>();

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = app_state.get_db_connection().await.into_db_error()?;

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                diesel::delete(validators::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;
                
                diesel::delete(governance_proposals::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(governance_votes::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(pos_rewards::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::insert_into(validators::table)
                    .values::<&Vec<ValidatorInsertDb>>(
                        &validators
                            .into_iter()
                            .map(|validator| {
                                ValidatorInsertDb::from_validator(validator, 1)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert validators in db")?;
                
                diesel::insert_into(governance_proposals::table)
                    .values::<&Vec<GovernanceProposalInsertDb>>(
                        &governance_proposals
                            .into_iter()
                            .map(|proposal| {
                                GovernanceProposalInsertDb::from_governance_proposal(proposal)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert proposals in db")?;

                diesel::insert_into(governance_votes::table)
                    .values::<&Vec<GovernanceProposalVoteInsertDb>>(
                        &governance_votes
                            .into_iter()
                            .map(|vote| {
                                GovernanceProposalVoteInsertDb::from_governance_vote(vote)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert goveranance votes in db")?;

                diesel::insert_into(pos_rewards::table)
                    .values::<&Vec<PosRewardInsertDb>>(
                        &rewards
                            .into_iter()
                            .map(|reward| {
                                PosRewardInsertDb::from_reward(reward, 1)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert pos rewards in db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()
}
