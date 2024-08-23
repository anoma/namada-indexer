// Manually create schema for views - see also https://github.com/diesel-rs/diesel/issues/1482
diesel::table! {
    balances (id) {
        id -> Int4,
        owner -> Varchar,
        token -> Varchar,
        raw_amount -> Numeric,
    }
}
