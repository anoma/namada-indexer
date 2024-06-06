use std::collections::{BTreeMap, HashSet};

use namada_parameters::storage as parameter_storage;
use namada_sdk::address::Address;
use namada_sdk::rpc::query_storage_value;
use namada_sdk::tendermint_rpc::HttpClient;
use namada_sdk::token;

use crate::appstate::AppState;
use crate::response::gas::GasCost;

#[derive(Clone)]
pub struct GasService {}

impl GasService {
    pub fn new(_app_state: AppState) -> Self {
        Self {}
    }

    pub async fn get_gas_table(&self, client: &HttpClient) -> HashSet<GasCost> {
        let key = parameter_storage::get_gas_cost_key();
        let gas_cost_table = query_storage_value::<
            HttpClient,
            BTreeMap<Address, token::Amount>,
        >(client, &key)
        .await
        .expect("Parameter should be defined.");

        let mut gas_table: HashSet<GasCost> = HashSet::new();

        for (token, gas_cost) in gas_cost_table {
            gas_table.insert(GasCost {
                token_address: token.to_string(),
                amount: gas_cost.to_string_native(),
            });
        }

        gas_table
    }
}
