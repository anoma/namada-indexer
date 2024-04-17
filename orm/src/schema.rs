// @generated automatically by Diesel CLI.

diesel::table! {
    tx_crawler_state (id) {
        id -> Int4,
        height -> Int4,
        epoch -> Int4,
    }
}
