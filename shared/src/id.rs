use std::fmt::Display;

use namada_core::hash::Hash as NamadaHash;
use serde::{Deserialize, Serialize};
use tendermint::{
    account::Id as TendermintAccountId, block::Id as TendermintBlockId,
    AppHash as TendermintAppHash, Hash as TendermintHash,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Id {
    Account(String),
    Hash(String),
}

impl Default for Id {
    fn default() -> Self {
        Self::Hash("".to_owned())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Account(id) => write!(f, "{}", id.to_lowercase()),
            Id::Hash(id) => write!(f, "{}", id.to_lowercase()),
        }
    }
}

impl From<TendermintBlockId> for Id {
    fn from(value: TendermintBlockId) -> Self {
        Self::Hash(value.hash.to_string())
    }
}

impl From<TendermintHash> for Id {
    fn from(value: TendermintHash) -> Self {
        Self::Hash(value.to_string())
    }
}

impl From<TendermintAppHash> for Id {
    fn from(value: TendermintAppHash) -> Self {
        Self::Hash(value.to_string())
    }
}

impl From<&TendermintAccountId> for Id {
    fn from(value: &TendermintAccountId) -> Self {
        Self::Account(value.to_string())
    }
}

impl From<NamadaHash> for Id {
    fn from(value: NamadaHash) -> Self {
        Self::Hash(value.to_string().to_lowercase())
    }
}
