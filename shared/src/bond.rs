use crate::{balance::Amount, block::Epoch, id::Id};

//TODO: maybe reuse bond with Option<Amount> instead of Amount
#[derive(Debug, Clone)]
pub struct BondAddresses {
    pub source: Id,
    pub target: Id,
}

#[derive(Debug, Clone)]
pub struct Bond {
    pub source: Id,
    pub target: Id,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct Bonds {
    pub epoch: Epoch,
    pub values: Vec<Bond>,
}
