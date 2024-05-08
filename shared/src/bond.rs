use crate::{
    balance::{Address, Amount},
    block::Epoch,
};

#[derive(Debug, Clone)]
pub struct Bond {
    pub source: Address,
    pub target: Address,
    pub amount: Amount,
}

#[derive(Debug, Clone)]
pub struct Bonds {
    pub epoch: Epoch,
    pub bonds: Vec<Bond>,
}
