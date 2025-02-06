use crate::appstate::AppState;
use crate::error::pgf::PgfError;
use crate::repository::pgf::{PgfRepo, PgfRepoTrait};
use crate::response::pgf::{PaymentKind, PaymentRecurrence, PgfPayment};

#[derive(Clone)]
pub struct PgfService {
    pgf_repo: PgfRepo,
}

impl PgfService {
    pub fn new(app_state: AppState) -> Self {
        Self {
            pgf_repo: PgfRepo::new(app_state.clone()),
        }
    }

    pub async fn get_all_pgf_payments(
        &self,
        page: u64,
    ) -> Result<(Vec<PgfPayment>, u64, u64), PgfError> {
        let (payments, total_pages, total_items) = self
            .pgf_repo
            .get_pgf_continuous_payments(page as i64)
            .await
            .map_err(PgfError::Database)?;

        let payments = payments
            .into_iter()
            .map(|payment| PgfPayment {
                payment_recurrence: PaymentRecurrence::from(
                    payment.payment_recurrence,
                ),
                proposal_id: payment.proposal_id,
                payment_kind: PaymentKind::from(payment.payment_kind),
                receipient: payment.receipient,
                amount: payment.amount.to_string(),
            })
            .collect();

        Ok((payments, total_pages as u64, total_items as u64))
    }

    pub async fn find_pfg_payment_by_proposal_id(
        &self,
        proposal_id: u64,
    ) -> Result<Option<PgfPayment>, PgfError> {
        let payment = self
            .pgf_repo
            .find_pgf_payment_by_proposal_id(proposal_id as i32)
            .await
            .map_err(PgfError::Database)?
            .map(|payment| PgfPayment {
                payment_recurrence: PaymentRecurrence::from(
                    payment.payment_recurrence,
                ),
                proposal_id: payment.proposal_id,
                payment_kind: PaymentKind::from(payment.payment_kind),
                receipient: payment.receipient,
                amount: payment.amount.to_string(),
            });

        Ok(payment)
    }
}
