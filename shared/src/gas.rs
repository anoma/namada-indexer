use crate::balance::Amount;

#[derive(Clone, Debug)]
pub struct GasPrice {
    pub token: String,
    pub amount: Amount,
}
