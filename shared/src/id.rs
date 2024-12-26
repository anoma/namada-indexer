use std::fmt::Display;
use std::str::FromStr;

use namada_sdk::address::Address as NamadaAddress;
use namada_sdk::hash::Hash as NamadaHash;
use namada_sdk::key::common;
use serde::{Deserialize, Serialize};
use tendermint::account::Id as TendermintAccountId;
use tendermint::block::Id as TendermintBlockId;
use tendermint::{AppHash as TendermintAppHash, Hash as TendermintHash};

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Deserialize, Serialize,
)]
pub enum Id {
    Account(String),
    IbcTrace(String),
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
            Id::IbcTrace(id) => write!(f, "{}", id.to_lowercase()),
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

impl From<&TendermintAppHash> for Id {
    fn from(value: &TendermintAppHash) -> Self {
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

impl From<NamadaAddress> for Id {
    fn from(value: NamadaAddress) -> Self {
        Self::Account(value.to_string().to_lowercase())
    }
}

impl From<Id> for NamadaAddress {
    fn from(value: Id) -> Self {
        match value {
            Id::Account(account) => NamadaAddress::from_str(&account).unwrap(),
            Id::Hash(_) => panic!(),
            Id::IbcTrace(s) => {
                namada_ibc::trace::convert_to_address(s).unwrap()
            }
        }
    }
}

impl From<common::PublicKey> for Id {
    fn from(value: common::PublicKey) -> Self {
        Id::Account(value.to_string())
    }
}
