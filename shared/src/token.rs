use std::fmt::Display;

use serde::Serialize;

use crate::id::Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct IbcToken {
    pub address: Id,
    pub trace: Id,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
// FIXME: review all the usage of this, I believe it's wrong, we use it even when we are not sure that we are dealing with the native token
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
