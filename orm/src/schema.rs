// @generated automatically by Diesel CLI.

diesel::table! {
    balances (id) {
        id -> Int4,
        address -> Varchar,
        amount -> Numeric,
    }
}

diesel::table! {
    nam_balances (id) {
        id -> Int4,
        address -> Varchar,
        amount -> Varchar,
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
    balances,
    nam_balances,
    tx_crawler_state,
);
