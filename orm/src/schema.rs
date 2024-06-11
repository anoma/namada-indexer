// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "governance_kind"))]
    pub struct GovernanceKind;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "governance_result"))]
    pub struct GovernanceResult;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "governance_tally_type"))]
    pub struct GovernanceTallyType;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "transaction_kind"))]
    pub struct TransactionKind;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "transaction_result"))]
    pub struct TransactionResult;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "validator_state"))]
    pub struct ValidatorState;

    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "vote_kind"))]
    pub struct VoteKind;
}

diesel::table! {
    balances (id) {
        id -> Int4,
        owner -> Varchar,
        token -> Varchar,
        raw_amount -> Varchar,
    }
}

diesel::table! {
    block_crawler_state (id) {
        id -> Int4,
        height -> Int4,
        epoch -> Int4,
        timestamp -> Int8,
    }
}

diesel::table! {
    bonds (id) {
        id -> Int4,
        address -> Varchar,
        validator_id -> Int4,
        raw_amount -> Varchar,
    }
}

diesel::table! {
    chain_parameters (id) {
        id -> Int4,
        unbonding_length -> Int4,
        pipeline_length -> Int4,
        epochs_per_year -> Int4,
        min_num_of_blocks -> Int4,
        min_duration -> Int4,
        apr -> Varchar,
        native_token_address -> Varchar,
    }
}

diesel::table! {
    epoch_crawler_state (id) {
        id -> Int4,
        epoch -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::GovernanceKind;
    use super::sql_types::GovernanceTallyType;
    use super::sql_types::GovernanceResult;

    governance_proposals (id) {
        id -> Int4,
        content -> Varchar,
        data -> Nullable<Varchar>,
        kind -> GovernanceKind,
        tally_type -> GovernanceTallyType,
        author -> Varchar,
        start_epoch -> Int4,
        end_epoch -> Int4,
        activation_epoch -> Int4,
        result -> GovernanceResult,
        yay_votes -> Varchar,
        nay_votes -> Varchar,
        abstain_votes -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::VoteKind;

    governance_votes (id) {
        id -> Int4,
        kind -> VoteKind,
        voter_address -> Varchar,
        proposal_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionKind;
    use super::sql_types::TransactionResult;

    inner_transactions (id) {
        #[max_length = 32]
        id -> Varchar,
        #[max_length = 32]
        wrapper_id -> Varchar,
        kind -> TransactionKind,
        data -> Nullable<Varchar>,
        memo -> Nullable<Varchar>,
        exit_code -> TransactionResult,
    }
}

diesel::table! {
    pos_rewards (id) {
        id -> Int4,
        owner -> Varchar,
        validator_id -> Int4,
        raw_amount -> Varchar,
    }
}

diesel::table! {
    revealed_pk (id) {
        id -> Int4,
        address -> Varchar,
        pk -> Varchar,
    }
}

diesel::table! {
    unbonds (id) {
        id -> Int4,
        address -> Varchar,
        validator_id -> Int4,
        raw_amount -> Varchar,
        withdraw_epoch -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ValidatorState;

    validators (id) {
        id -> Int4,
        namada_address -> Varchar,
        voting_power -> Int4,
        max_commission -> Varchar,
        commission -> Varchar,
        name -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        discord_handle -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        state -> ValidatorState,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionResult;

    wrapper_transactions (id) {
        #[max_length = 32]
        id -> Varchar,
        fee_amount_per_gas_unit_amount -> Varchar,
        fee_payer -> Varchar,
        fee_token -> Varchar,
        gas_limit -> Varchar,
        block_height -> Int4,
        exit_code -> TransactionResult,
        atomic -> Bool,
    }
}

diesel::joinable!(bonds -> validators (validator_id));
diesel::joinable!(governance_votes -> governance_proposals (proposal_id));
diesel::joinable!(inner_transactions -> wrapper_transactions (wrapper_id));
diesel::joinable!(pos_rewards -> validators (validator_id));
diesel::joinable!(unbonds -> validators (validator_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    block_crawler_state,
    bonds,
    chain_parameters,
    epoch_crawler_state,
    governance_proposals,
    governance_votes,
    inner_transactions,
    pos_rewards,
    revealed_pk,
    unbonds,
    validators,
    wrapper_transactions,
);
