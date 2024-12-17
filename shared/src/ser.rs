use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use namada_core::address::Address;
use namada_core::masp::MaspTxId;
use namada_sdk::borsh::BorshSerializeExt;
use namada_sdk::ibc::IbcMessage as NamadaIbcMessage;
use namada_sdk::token::{
    Account as NamadaAccount, DenominatedAmount as NamadaDenominatedAmount,
    Transfer as NamadaTransfer,
};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use subtle_encoding::hex;

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
                            NamadaDenominatedAmount::from_str(amount)
                                .expect("Cannot parse DenominatedAmount"),
                        )
                    })
                    .collect(),
            )
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TransparentTransfer {
    /// Sources of this transfer
    pub sources: AccountsMap,
    /// Targets of this transfer
    pub targets: AccountsMap,
    /// Hash of tx section that contains the MASP transaction
    pub shielded_section_hash: Option<MaspTxId>,
}

impl From<NamadaTransfer> for TransparentTransfer {
    fn from(transfer: NamadaTransfer) -> Self {
        let sources = AccountsMap(transfer.sources);
        let targets = AccountsMap(transfer.targets);
        let shielded_section_hash = transfer.shielded_section_hash;

        TransparentTransfer {
            sources,
            targets,
            shielded_section_hash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IbcMessage<Transfer>(pub NamadaIbcMessage<Transfer>);

impl Serialize for IbcMessage<NamadaTransfer> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            NamadaIbcMessage::Transfer(ibc_transfer) => {
                let mut state =
                    serializer.serialize_struct("IbcTransfer", 2)?;

                state.serialize_field("message", &ibc_transfer.message)?;
                state.serialize_field("transfer", &ibc_transfer.transfer)?;

                state.end()
            }
            NamadaIbcMessage::NftTransfer(ibc_nft_transfer) => {
                let mut state =
                    serializer.serialize_struct("IbcNftTransfer", 2)?;

                state.serialize_field(
                    "nft_message",
                    &ibc_nft_transfer.message,
                )?;
                state
                    .serialize_field("transfer", &ibc_nft_transfer.transfer)?;

                state.end()
            }
            NamadaIbcMessage::Envelope(data) => {
                let mut state =
                    serializer.serialize_struct("IbcEnvelope", 1)?;

                // todo: implement this bs :(

                state.serialize_field(
                    "data",
                    &String::from_utf8_lossy(&hex::encode(
                        data.serialize_to_vec(),
                    ))
                    .into_owned(),
                )?;

                state.end()
            }
        }
    }
}
