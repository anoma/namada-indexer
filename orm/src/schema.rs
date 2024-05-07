// @generated automatically by Diesel CLI.

diesel::table! {
    balances (id) {
        id -> Int4,
        owner -> Varchar,
        token -> Varchar,
        height -> Int4,
        raw_amount -> Numeric,
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
    epoch_crawler_state (id) {
        id -> Int4,
        epoch -> Int4,
    }
}

diesel::table! {
    validators (id) {
        id -> Int4,
        namada_address -> Varchar,
        voting_power -> Int4,
        max_commission -> Varchar,
        commission -> Varchar,
        email -> Varchar,
        website -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        discord_handle -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        epoch -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    block_crawler_state,
    epoch_crawler_state,
    validators,
);
