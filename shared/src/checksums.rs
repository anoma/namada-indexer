use bimap::BiMap;
use namada_sdk::tx::{
    TX_BOND_WASM, TX_CHANGE_COMMISSION_WASM, TX_CHANGE_METADATA_WASM,
    TX_CLAIM_REWARDS_WASM, TX_IBC_WASM, TX_INIT_PROPOSAL, TX_REDELEGATE_WASM,
    TX_REVEAL_PK, TX_TRANSFER_WASM, TX_UNBOND_WASM, TX_VOTE_PROPOSAL,
    TX_WITHDRAW_WASM,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Checksums(BiMap<String, String>);

impl Checksums {
    pub fn get_name_by_id(&self, hash: &str) -> Option<String> {
        self.0.get_by_right(hash).map(|data| data.to_owned())
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<String> {
        self.0.get_by_left(name).map(|data| data.to_owned())
    }

    pub fn add(&mut self, key: String, value: String) {
        let key = key.strip_suffix(".wasm").unwrap().to_owned();
        self.0.insert(key, value);
    }

    pub fn add_with_ext(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn code_paths() -> Vec<String> {
        vec![
            TX_REVEAL_PK.to_string(),
            TX_REVEAL_PK.to_string(),
            TX_TRANSFER_WASM.to_string(),
            TX_BOND_WASM.to_string(),
            TX_REDELEGATE_WASM.to_string(),
            TX_UNBOND_WASM.to_string(),
            TX_WITHDRAW_WASM.to_string(),
            TX_CLAIM_REWARDS_WASM.to_string(),
            TX_VOTE_PROPOSAL.to_string(),
            TX_INIT_PROPOSAL.to_string(),
            TX_CHANGE_METADATA_WASM.to_string(),
            TX_CHANGE_COMMISSION_WASM.to_string(),
            TX_IBC_WASM.to_string(),
        ]
    }
}
