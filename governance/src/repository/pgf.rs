use anyhow::Context;
use diesel::query_dsl::methods::FilterDsl;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use orm::balances::BalanceChangesInsertDb;
use orm::pgf::PublicGoodFundingPaymentInsertDb;
use orm::schema::{balance_changes, public_good_funding};
use shared::block::BlockHeight;
use shared::id::Id;
use shared::pgf::{PaymentRecurrence, PgfPayment};

pub fn update_pgf(
    transaction_conn: &mut PgConnection,
    pgf_payments: Vec<PgfPayment>,
    native_token: Id,
    block_height: BlockHeight,
) -> anyhow::Result<()> {
    diesel::insert_into(balance_changes::table)
        .values::<Vec<BalanceChangesInsertDb>>(
            pgf_payments
                .clone()
                .into_iter()
                .filter_map(|payment| {
                    if matches!(payment.recurrence, PaymentRecurrence::Retro)
                        || (matches!(
                            payment.recurrence,
                            PaymentRecurrence::Continous
                        ) && matches!(
                            payment.action,
                            Some(shared::pgf::PgfAction::Add)
                        ))
                    {
                        Some(BalanceChangesInsertDb::from_pgf_retro(
                            payment,
                            native_token.clone(),
                            block_height,
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            balance_changes::columns::owner,
            balance_changes::columns::token,
        ))
        .do_update()
        .set(
            balance_changes::columns::raw_amount
                .eq(balance_changes::columns::raw_amount
                    + excluded(balance_changes::columns::raw_amount)),
        )
        .execute(transaction_conn)
        .context("Failed to update balance_changes in db")?;

    diesel::insert_into(public_good_funding::table)
        .values::<Vec<PublicGoodFundingPaymentInsertDb>>(
            pgf_payments
                .clone()
                .into_iter()
                .filter(|payment| {
                    matches!(payment.recurrence, PaymentRecurrence::Retro)
                        || (matches!(
                            payment.recurrence,
                            PaymentRecurrence::Continous
                        ) && matches!(
                            payment.action,
                            Some(shared::pgf::PgfAction::Add)
                        ))
                })
                .map(PublicGoodFundingPaymentInsertDb::from_pgf_payment)
                .collect::<Vec<_>>(),
        )
        .on_conflict((
            public_good_funding::columns::proposal_id,
            public_good_funding::columns::receipient,
        ))
        .do_nothing()
        .execute(transaction_conn)
        .context("Failed to update balance_changes in db")?;

    for payment in pgf_payments.into_iter().filter(|payment| {
        matches!(payment.recurrence, PaymentRecurrence::Continous)
            && matches!(payment.action, Some(shared::pgf::PgfAction::Remove))
    }) {
        diesel::delete(
            public_good_funding::table.filter(
                public_good_funding::columns::receipient
                    .eq(payment.receipient.to_string()),
            ),
        )
        .execute(transaction_conn)?;
    }

    anyhow::Ok(())
}
