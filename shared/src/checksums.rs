use bimap::BiMap;
use namada_sdk::tx::{
    TX_BOND_WASM, TX_CLAIM_REWARDS_WASM, TX_INIT_PROPOSAL, TX_REDELEGATE_WASM,
    TX_TRANSFER_WASM, TX_UNBOND_WASM, TX_VOTE_PROPOSAL, TX_WITHDRAW_WASM,
};
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
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

    pub fn code_paths() -> Vec<String> {
        vec![
            TX_TRANSFER_WASM.to_string(),
            TX_BOND_WASM.to_string(),
            TX_REDELEGATE_WASM.to_string(),
            TX_UNBOND_WASM.to_string(),
            TX_WITHDRAW_WASM.to_string(),
            TX_CLAIM_REWARDS_WASM.to_string(),
            TX_VOTE_PROPOSAL.to_string(),
            TX_INIT_PROPOSAL.to_string(),
        ]
    }
}
