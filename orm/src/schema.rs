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
    epoch_crawler_state (id) {
        id -> Int4,
        epoch -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::GovernanceKind;
    use super::sql_types::GovernanceResult;

    governance_proposals (id) {
        id -> Int4,
        content -> Varchar,
        data -> Nullable<Varchar>,
        kind -> GovernanceKind,
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
    pos_rewards (id) {
        id -> Int4,
        owner -> Varchar,
        validator_id -> Int4,
        raw_amount -> Varchar,
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
    validators (id) {
        id -> Int4,
        namada_address -> Varchar,
        voting_power -> Int4,
        max_commission -> Varchar,
        commission -> Varchar,
        email -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        discord_handle -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
    }
}

diesel::joinable!(bonds -> validators (validator_id));
diesel::joinable!(governance_votes -> governance_proposals (proposal_id));
diesel::joinable!(pos_rewards -> validators (validator_id));
diesel::joinable!(unbonds -> validators (validator_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    block_crawler_state,
    bonds,
    epoch_crawler_state,
    governance_proposals,
    governance_votes,
    pos_rewards,
    unbonds,
    validators,
);
