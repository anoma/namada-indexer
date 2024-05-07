use crate::id::Id;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct BalanceChange {
    pub address: Id,
    pub token: Id
}

impl BalanceChange {
    pub fn new(address: Id, token: Id) -> Self {
        Self {
            address,
            token,
        }
    }
}