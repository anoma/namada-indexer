use std::collections::HashSet;

use anyhow::Context;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use orm::pgf::{PaymentRecurrenceDb, PublicGoodFundingPaymentDb};
use orm::schema::public_good_funding;
use shared::block::Epoch;
use shared::id::Id;
use shared::token::Token;
use shared::utils::BalanceChange;

pub fn get_pgf_receipients_balance_changes(
    transaction_conn: &mut PgConnection,
    current_epoch: Epoch,
    native_token: Id,
) -> anyhow::Result<HashSet<BalanceChange>> {
    let epoch = current_epoch - 1;

    public_good_funding::table
        .filter(
            public_good_funding::dsl::payment_recurrence
                .eq(PaymentRecurrenceDb::Continous)
                .and(
                    public_good_funding::dsl::last_paid_epoch.eq(epoch as i32),
                ),
        )
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
