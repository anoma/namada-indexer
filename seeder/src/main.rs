use anyhow::Context;
use clap::Parser;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::balances::BalanceChangesInsertDb;
use orm::bond::BondInsertDb;
use orm::governance_proposal::{
    GovernanceProposalInsertDb, GovernanceProposalUpdateStatusDb,
};
use orm::governance_votes::GovernanceProposalVoteInsertDb;
use orm::pos_rewards::PosRewardInsertDb;
use orm::schema::{
    balance_changes, bonds, governance_proposals, governance_votes,
    pos_rewards, unbonds, validators,
};
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
use shared::unbond::Unbond;
use shared::validator::Validator;
use shared::vote::GovernanceVote;

#[tokio::main]
async fn main() -> anyhow::Result<(), MainError> {
    let config = AppConfig::parse();

    config.log.init();

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
            Reward::fake(index)
        })
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

                diesel::delete(balance_changes::table)
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
                    .execute(transaction_conn)
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
                                let validator_id = reward.delegation_pair.validator_address.to_string().parse::<i32>().unwrap();
                                let epoch = reward.epoch;
                                PosRewardInsertDb::from_reward(reward, validator_id, epoch)
                            })
                            .collect::<Vec<_>>(),
                    )
                    .execute(transaction_conn)
                    .context("Failed to insert pos rewards in db")?;

                diesel::insert_into(balance_changes::table)
                    .values::<&Vec<BalanceChangesInsertDb>>(
                        &balances
                            .into_iter()
                            .map(|balance| {
                                BalanceChangesInsertDb::from_balance(balance)
                            })
                            .collect::<Vec<_>>(),
                    )
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

                anyhow::Ok(())
            })
    })
    .await
    .context_db_interact_error()
    .into_db_error()?
    .context("Commit block db transaction error")
    .into_db_error()
}
