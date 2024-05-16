use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::balances::BalancesInsertDb;
use orm::bond::BondInsertDb;
use orm::governance_proposal::{GovernanceProposalInsertDb, GovernanceProposalUpdateStatusDb};
use orm::governance_votes::GovernanceProposalVoteInsertDb;
use orm::pos_rewards::PosRewardInsertDb;
use orm::schema::{
    balances, bonds, governance_proposals, governance_votes, pos_rewards,
    unbonds, validators,
};
use orm::unbond::UnbondInsertDb;
use orm::validators::{ValidatorDb, ValidatorInsertDb};
use rand::Rng;
use seeder::config::AppConfig;
use seeder::state::AppState;
use shared::balance::Balance;
use shared::bond::{Bond, Bonds};
use shared::error::{AsDbError, ContextDbInteractError, MainError};
use shared::proposal::{GovernanceProposal, GovernanceProposalStatus};
use shared::rewards::Reward;
use shared::unbond::{Unbond, Unbonds};
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
        .map(GovernanceProposal::fake)
        .collect::<Vec<GovernanceProposal>>();

    let governance_proposals_status = (0..config.total_proposals)
        .map(GovernanceProposalStatus::fake)
        .collect::<Vec<GovernanceProposalStatus>>();

    let governance_votes = (0..config.total_votes)
        .map(|_| {
            let proposal_id =
                rand::thread_rng().gen_range(0..config.total_proposals);
            GovernanceVote::fake(proposal_id)
        })
        .collect::<Vec<GovernanceVote>>();

    let bonds = (0..10)
        .map(|_| {
            let index =
                rand::thread_rng().gen_range(0..config.total_validators);
            let validator = validators.get(index as usize).unwrap();
            Bond::fake(validator.address.clone())
        })
        .collect::<Vec<Bond>>();

    let unbonds = (0..10)
        .map(|_| {
            let index =
                rand::thread_rng().gen_range(0..config.total_validators);
            let validator = validators.get(index as usize).unwrap();
            Unbond::fake(validator.address.clone())
        })
        .collect::<Vec<Unbond>>();

    let rewards = (0..config.total_rewards)
        .map(|_| Reward::fake())
        .collect::<Vec<Reward>>();

    let balances = (0..config.total_balances)
        .map(|_| Balance::fake())
        .collect::<Vec<Balance>>();

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = app_state.get_db_connection().await.into_db_error()?;

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                diesel::delete(governance_votes::table)
                .execute(transaction_conn)
                .context("Failed to remove all validators")?;

                diesel::delete(governance_proposals::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(pos_rewards::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(bonds::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(unbonds::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(balances::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                    diesel::delete(validators::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::insert_into(validators::table)
                    .values::<&Vec<ValidatorInsertDb>>(
                        &validators
                            .into_iter()
                            .map(|validator| {
                                ValidatorInsertDb::from_validator(validator)
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

                for proposal_status in governance_proposals_status {
                    diesel::update(governance_proposals::table.find(proposal_status.id as i32))
                        .set::<GovernanceProposalUpdateStatusDb>(proposal_status.into())
                        .execute(transaction_conn)?;
                }

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

                diesel::insert_into(balances::table)
                    .values::<&Vec<BalancesInsertDb>>(
                        &balances
                            .into_iter()
                            .map(|balance| {
                                BalancesInsertDb::from_balance(balance)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .on_conflict_do_nothing()
                    .execute(transaction_conn)
                    .context("Failed to insert pos rewards in db")?;

                diesel::insert_into(bonds::table)
                .values::<&Vec<BondInsertDb>>(
                    &bonds
                    .into_iter()
                    .map(|bond| {
                        let validator: ValidatorDb = validators::table
                            .filter(validators::namada_address.eq(&bond.target.to_string()))
                            .select(ValidatorDb::as_select())
                            .first(transaction_conn)
                            .expect("Failed to get validator");

                        BondInsertDb::from_bond(bond, validator.id)
                    })
                    .collect::<Vec<_>>())
                .on_conflict((bonds::columns::validator_id, bonds::columns::address, bonds::columns::epoch))
                .do_update()
                .set(orm::schema::bonds::columns::raw_amount
                    .eq(excluded(orm::schema::bonds::columns::raw_amount)))
                .execute(transaction_conn)
                .context("Failed to update bonds in db")?;

                diesel::insert_into(unbonds::table)
                .values::<&Vec<UnbondInsertDb>>(
                    &unbonds
                    .into_iter()
                    .map(|unbond| {
                        let validator: ValidatorDb = validators::table
                            .filter(validators::namada_address.eq(&unbond.target.to_string()))
                            .select(ValidatorDb::as_select())
                            .first(transaction_conn)
                            .expect("Failed to get validator");

                        UnbondInsertDb::from_unbond(unbond, validator.id)
                    })
                    .collect::<Vec<_>>())
                .on_conflict((unbonds::columns::validator_id, unbonds::columns::address, unbonds::columns::epoch))
                .do_update()
                .set((unbonds::columns::raw_amount.eq(excluded(unbonds::columns::raw_amount)),
                      unbonds::columns::withdraw_epoch.eq(excluded(unbonds::columns::withdraw_epoch))))
                .execute(transaction_conn)
                .context("Failed to update unbonds in db")?;

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()
}
