use shared::{balance::Amount, id::Id, token::Token};

#[derive(Debug, Clone)]
pub struct Balance {
    pub owner: Id,
    pub token: Token,
    pub amount: Amount,
}
