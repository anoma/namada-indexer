use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use namada_core::address::Address;
use namada_core::masp::TxId;
use namada_sdk::token::{
    Account as NamadaAccount, DenominatedAmount as NamadaDenominatedAmount,
    Transfer as NamadaTransfer,
};

#[derive(Debug, Clone)]
pub struct AccountsMap(pub BTreeMap<NamadaAccount, NamadaDenominatedAmount>);

impl Serialize for AccountsMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.iter().map(|(k, v)| {
            HashMap::from([
                ("owner", k.owner.encode()),
                ("token", k.token.encode()),
                ("amount", v.to_string_precise()),
            ])
        }))
    }
}

impl<'de> Deserialize<'de> for AccountsMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <Vec<BTreeMap<String, String>> as Deserialize>::deserialize(
            deserializer,
        )
        .map(|v| {
            AccountsMap(
                v.into_iter()
                    .map(|val| {
                        let owner =
                            val.get("owner").expect("Cannot find owner");
                        let token =
                            val.get("token").expect("Cannot find token");
                        let amount =
                            val.get("amount").expect("Cannot find amount");

                        (
                            NamadaAccount {
                                owner: Address::decode(owner)
                                    .expect("Cannot parse Address for owner"),
                                token: Address::decode(token)
                                    .expect("Cannot parse Address for token"),
                            },
                            NamadaDenominatedAmount::from_str(&amount)
                                .expect("Cannot parse DenominatedAmount"),
                        )
                    })
                    .collect(),
            )
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transfer {
    /// Sources of this transfer
    pub sources: AccountsMap,
    /// Targets of this transfer
    pub targets: AccountsMap,
    /// Hash of tx section that contains the MASP transaction
    pub shielded_section_hash: Option<TxId>,
}

impl From<NamadaTransfer> for Transfer {
    fn from(transfer: NamadaTransfer) -> Self {
        let sources = AccountsMap(transfer.sources);
        let targets = AccountsMap(transfer.targets);
        let shielded_section_hash = transfer.shielded_section_hash;

        Transfer {
            sources,
            targets,
            shielded_section_hash,
        }
    }
}
