// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "crawler_name"))]
    pub struct CrawlerName;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "governance_kind"))]
    pub struct GovernanceKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "governance_result"))]
    pub struct GovernanceResult;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "governance_tally_type"))]
    pub struct GovernanceTallyType;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ibc_status"))]
    pub struct IbcStatus;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_kind"))]
    pub struct PaymentKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_recurrence"))]
    pub struct PaymentRecurrence;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "token_type"))]
    pub struct TokenType;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_kind"))]
    pub struct TransactionKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_result"))]
    pub struct TransactionResult;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "validator_state"))]
    pub struct ValidatorState;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vote_kind"))]
    pub struct VoteKind;
}

diesel::table! {
    balance_changes (id) {
        id -> Int4,
        height -> Int4,
        owner -> Varchar,
        #[max_length = 64]
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
        max_block_time -> Int4,
        apr -> Varchar,
        native_token_address -> Varchar,
        chain_id -> Varchar,
        genesis_time -> Int8,
        epoch_switch_blocks_delay -> Int4,
        checksums -> Jsonb,
        cubic_slashing_window_length -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CrawlerName;

    crawler_state (name) {
        name -> CrawlerName,
        last_processed_block -> Nullable<Int4>,
        first_block_in_epoch -> Nullable<Int4>,
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
    use super::sql_types::IbcStatus;

    ibc_ack (id) {
        id -> Varchar,
        tx_hash -> Varchar,
        timeout -> Int4,
        status -> Nullable<IbcStatus>,
    }
}

diesel::table! {
    ibc_token (address) {
        #[max_length = 45]
        address -> Varchar,
        ibc_trace -> Varchar,
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
    use diesel::sql_types::*;
    use super::sql_types::PaymentRecurrence;
    use super::sql_types::PaymentKind;

    public_good_funding (id) {
        id -> Int4,
        proposal_id -> Int4,
        payment_recurrence -> PaymentRecurrence,
        payment_kind -> PaymentKind,
        receipient -> Varchar,
        amount -> Numeric,
<<<<<<< HEAD
=======
        last_paid_epoch -> Int4,
>>>>>>> 3e415012 (parsing ack tx)
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
    use diesel::sql_types::*;
    use super::sql_types::TokenType;

    token (address) {
        #[max_length = 45]
        address -> Varchar,
        token_type -> TokenType,
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

diesel::joinable!(balance_changes -> token (token));
diesel::joinable!(bonds -> validators (validator_id));
diesel::joinable!(governance_votes -> governance_proposals (proposal_id));
diesel::joinable!(ibc_token -> token (address));
diesel::joinable!(inner_transactions -> wrapper_transactions (wrapper_id));
diesel::joinable!(pos_rewards -> validators (validator_id));
diesel::joinable!(public_good_funding -> governance_proposals (proposal_id));
diesel::joinable!(unbonds -> validators (validator_id));

diesel::allow_tables_to_appear_in_same_query!(
    balance_changes,
    bonds,
    chain_parameters,
    crawler_state,
    gas,
    gas_price,
    governance_proposals,
    governance_votes,
    ibc_ack,
    ibc_token,
    inner_transactions,
    pos_rewards,
    public_good_funding,
    revealed_pk,
    token,
    unbonds,
    validators,
    wrapper_transactions,
);
