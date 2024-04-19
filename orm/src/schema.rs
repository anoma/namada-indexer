// @generated automatically by Diesel CLI.

diesel::table! {
    balances (id) {
        id -> Int4,
        owner -> Varchar,
        token -> Varchar,
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

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    block_crawler_state,
);
