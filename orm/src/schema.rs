// @generated automatically by Diesel CLI.

diesel::table! {
    block_crawler_state (id) {
        id -> Int4,
        height -> Int4,
        epoch -> Int4,
    }
}

diesel::table! {
    nam_balances (id) {
        id -> Int4,
        owner -> Varchar,
        raw_amount -> Numeric,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    block_crawler_state,
    nam_balances,
);
