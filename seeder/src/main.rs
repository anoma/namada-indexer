use anyhow::Context;
use clap::Parser;
use clap_verbosity_flag::LevelFilter;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::balances::BalancesInsertDb;
use orm::bond::BondInsertDb;
use orm::governance_proposal::{
    GovernanceProposalInsertDb, GovernanceProposalUpdateStatusDb,
};
use orm::governance_votes::GovernanceProposalVoteInsertDb;
use orm::pos_rewards::PosRewardInsertDb;
use orm::schema::{
    balances, bonds, governance_proposals, governance_votes,
    inner_transactions, pos_rewards, unbonds, validators, wrapper_transactions,
};
use orm::transactions::{InnerTransactionInsertDb, WrapperTransactionInsertDb};
use orm::unbond::UnbondInsertDb;
use orm::validators::{ValidatorDb, ValidatorInsertDb};
use rand::Rng;
use seeder::config::AppConfig;
use seeder::state::AppState;
use shared::balance::Balance;
use shared::bond::Bond;
use shared::error::{AsDbError, ContextDbInteractError, MainError};
use shared::proposal::{
    GovernanceProposal, GovernanceProposalStatus, TallyType,
};
use shared::rewards::Reward;
use shared::transaction::{InnerTransaction, WrapperTransaction};
use shared::unbond::Unbond;
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

    let governance_proposals_with_tally = governance_proposals
        .iter()
        .cloned()
        .map(|proposal| {
            let tally_type = TallyType::fake();
            (proposal, tally_type)
        })
        .collect::<Vec<(GovernanceProposal, TallyType)>>();

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
        .map(|_| {
            let index =
                rand::thread_rng().gen_range(0..config.total_validators);
            let validator = validators.get(index as usize).unwrap();
            Reward::fake(validator.address.clone())
        })
        .collect::<Vec<Reward>>();

    let balances = (0..config.total_balances)
        .map(|_| Balance::fake())
        .collect::<Vec<Balance>>();

    let wrapper_txs = (0..10)
        .map(|_| WrapperTransaction::fake())
        .collect::<Vec<WrapperTransaction>>();

    let inner_txs = (0..50)
        .map(|_| {
            let index = rand::thread_rng().gen_range(0..wrapper_txs.len());
            let wrapper_tx = wrapper_txs[index].clone();
            InnerTransaction::fake(wrapper_tx.tx_id)
        })
        .collect::<Vec<_>>();

    let app_state = AppState::new(config.database_url).into_db_error()?;
    let conn = app_state.get_db_connection().await.into_db_error()?;

    conn.interact(move |conn| {
        conn.build_transaction()
            .read_write()
            .run(|transaction_conn| {
                diesel::delete(governance_votes::table)
                .execute(transaction_conn)
                .context("Failed to remove all votes")?;

                diesel::delete(governance_proposals::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all governance proposal")?;

                diesel::delete(pos_rewards::table)
                    .execute(transaction_conn)
                    .context("Failed to remove pos rewards")?;

                diesel::delete(bonds::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all bonds")?;

                diesel::delete(unbonds::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all unbonds")?;

                diesel::delete(balances::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all balances")?;

                diesel::delete(validators::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all validators")?;

                diesel::delete(inner_transactions::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all inner transactions")?;

                diesel::delete(wrapper_transactions::table)
                    .execute(transaction_conn)
                    .context("Failed to remove all wrapper transactions")?;

                diesel::insert_into(validators::table)
                    .values::<&Vec<ValidatorInsertDb>>(
                        &validators
                            .into_iter()
                            .map(|validator| {
                                ValidatorInsertDb::from_validator(validator)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .returning(validators::dsl::id)
                    .get_results::<i32>(transaction_conn)
                    .context("Failed to insert validators in db")?;

                diesel::insert_into(governance_proposals::table)
                    .values::<&Vec<GovernanceProposalInsertDb>>(
                        &governance_proposals_with_tally
                            .into_iter()
                            .map(|(proposal, tally_type)| {
                                GovernanceProposalInsertDb::from_governance_proposal(proposal, tally_type)
                            })
                            .collect::<Vec<_>>(),
                    )
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
                    .execute(transaction_conn)
                    .context("Failed to insert goveranance votes in db")?;

                diesel::insert_into(pos_rewards::table)
                    .values::<&Vec<PosRewardInsertDb>>(
                        &rewards
                            .into_iter()
                            .map(|reward| {
                                let validator: ValidatorDb = validators::table
                                    .filter(validators::namada_address.eq(&reward.delegation_pair.validator_address.to_string()))
                                    .select(ValidatorDb::as_select())
                                    .first(transaction_conn)
                                    .expect("Failed to get validator");
                                PosRewardInsertDb::from_reward(reward, validator.id)
                            })
                            .collect::<Vec<_>>(),
                    )
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
                    .execute(transaction_conn)
                    .context("Failed to balances rewards in db")?;

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
                .execute(transaction_conn)
                .context("Failed to update unbonds in db")?;

                diesel::insert_into(wrapper_transactions::table)
                .values::<&Vec<WrapperTransactionInsertDb>>(
                    &wrapper_txs
                    .into_iter()
                    .map(|wrapper_tx| {
                        WrapperTransactionInsertDb::from(wrapper_tx)
                    })
                    .collect::<Vec<_>>())
                .execute(transaction_conn)
                .context("Failed to update unbonds in db")?;

                diesel::insert_into(inner_transactions::table)
                .values::<&Vec<InnerTransactionInsertDb>>(
                    &inner_txs
                    .into_iter()
                    .map(|inner_tx| {
                        InnerTransactionInsertDb::from(inner_tx)
                    })
                    .collect::<Vec<_>>())
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
