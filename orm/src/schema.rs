// @generated automatically by Diesel CLI.

diesel::table! {
    nam_balances (id) {
        id -> Int4,
        address -> Varchar,
        raw_amount -> Numeric,
    }
}

diesel::table! {
    tx_crawler_state (id) {
        id -> Int4,
        height -> Int4,
        epoch -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    nam_balances,
    tx_crawler_state,
);
