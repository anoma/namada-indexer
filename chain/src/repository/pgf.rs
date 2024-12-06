use std::collections::HashSet;

use anyhow::Context;
use diesel::{PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use orm::pgf::PublicGoodFundingPaymentDb;
use orm::schema::public_good_funding;
use shared::id::Id;
use shared::token::Token;
use shared::utils::BalanceChange;

pub fn get_pgf_receipients_balance_changes(
    transaction_conn: &mut PgConnection,
    native_token: &Id,
) -> anyhow::Result<HashSet<BalanceChange>> {
    public_good_funding::table
        .select(PublicGoodFundingPaymentDb::as_select())
        .load(transaction_conn)
        .map(|data| {
            data.into_iter()
                .map(|payment| BalanceChange {
                    address: Id::Account(payment.receipient),
                    token: Token::Native(native_token.clone()),
                })
                .collect::<HashSet<_>>()
        })
        .context("Failed to update governance votes in db")
}
