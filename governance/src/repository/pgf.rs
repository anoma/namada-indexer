use anyhow::Context;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, RunQueryDsl,
};
use orm::pgf::{PaymentRecurrenceDb, PublicGoodFundingPaymentInsertDb};
use orm::schema::public_good_funding;
use shared::pgf::{PaymentRecurrence, PgfPayment};

pub fn update_pgf(
    transaction_conn: &mut PgConnection,
    pgf_payments: Vec<PgfPayment>,
) -> anyhow::Result<()> {
    diesel::insert_into(public_good_funding::table)
        .values::<Vec<PublicGoodFundingPaymentInsertDb>>(
            pgf_payments
                .clone()
                .into_iter()
                .filter(|payment| {
                    matches!(payment.recurrence, PaymentRecurrence::Retro)
                        || (matches!(
                            payment.recurrence,
                            PaymentRecurrence::Continuous
                        ) && matches!(
                            payment.action,
                            Some(shared::pgf::PgfAction::Add)
                        ))
                })
                .map(PublicGoodFundingPaymentInsertDb::from_pgf_payment)
                .collect::<Vec<_>>(),
        )
        .on_conflict_do_nothing()
        .execute(transaction_conn)
        .context("Failed to update balance_changes in db")?;

    for payment in pgf_payments.into_iter().filter(|payment| {
        matches!(payment.recurrence, PaymentRecurrence::Continuous)
            && matches!(payment.action, Some(shared::pgf::PgfAction::Remove))
    }) {
        diesel::delete(
            public_good_funding::table.filter(
                public_good_funding::dsl::receipient
                    .eq(payment.receipient.to_string())
                    .and(
                        public_good_funding::dsl::payment_recurrence
                            .eq(PaymentRecurrenceDb::Continuous),
                    ),
            ),
        )
        .execute(transaction_conn)?;
    }

    anyhow::Ok(())
}
