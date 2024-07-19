// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(
        diesel::query_builder::QueryId,
        std::fmt::Debug,
        diesel::sql_types::SqlType,
    )]
    #[diesel(postgres_type(name = "crawler_name"))]
    pub struct CrawlerName;

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
        raw_amount -> Numeric,
    }
}

diesel::table! {
    bonds (id) {
        id -> Int4,
        address -> Varchar,
        validator_id -> Int4,
        raw_amount -> Numeric,
        start -> Int4,
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
        chain_id -> Varchar,
        genesis_time -> Int8,
        epoch_switch_blocks_delay -> Int4,
        checksums -> Jsonb,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CrawlerName;

    crawler_state (name) {
        name -> CrawlerName,
        last_processed_block -> Nullable<Int4>,
        last_processed_epoch -> Nullable<Int4>,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionKind;

    gas (id) {
        id -> Int4,
        tx_kind -> TransactionKind,
        token -> Varchar,
        gas_limit -> Int4,
    }
}

diesel::table! {
    gas_price (token) {
        token -> Varchar,
        amount -> Numeric,
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
        #[max_length = 64]
        id -> Varchar,
        #[max_length = 64]
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
        raw_amount -> Numeric,
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
        raw_amount -> Numeric,
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
        #[max_length = 64]
        id -> Varchar,
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
    bonds,
    chain_parameters,
    crawler_state,
    gas,
    gas_price,
    governance_proposals,
    governance_votes,
    inner_transactions,
    pos_rewards,
    revealed_pk,
    unbonds,
    validators,
    wrapper_transactions,
);
