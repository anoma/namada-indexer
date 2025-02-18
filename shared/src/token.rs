use std::fmt::Display;

use bigdecimal::BigDecimal;
use serde::Serialize;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct IbcToken {
    pub address: Id,
    pub trace: Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Token {
    Ibc(IbcToken),
    Native(Id),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ibc(token) => write!(f, "{}", token.address),
            Token::Native(token) => write!(f, "{}", token),
        }
    }
}

#[derive(Debug)]
pub struct IbcRateLimit {
    /// Address of the token in Namada
    pub address: String,
    /// Epoch of the indexed rate limit
    pub epoch: u32,
    /// Throughput limit of token `address` at epoch `epoch`
    pub throughput_limit: BigDecimal,
}
