use std::collections::BTreeMap;

use namada_parameters::storage as parameter_storage;
use namada_sdk::{
    address::Address, rpc::query_storage_value, tendermint_rpc::HttpClient,
    token,
};

use crate::{appstate::AppState, response::gas::GasCost};

#[derive(Clone)]
pub struct GasService {}

impl GasService {
    pub fn new(_app_state: AppState) -> Self {
        Self {}
    }

    pub async fn get_gas_table(&self, client: &HttpClient) -> Vec<GasCost> {
        let key = parameter_storage::get_gas_cost_key();
        let gas_cost_table = query_storage_value::<
            HttpClient,
            BTreeMap<Address, token::Amount>,
        >(client, &key)
        .await
        .expect("Parameter should be defined.");

        let mut gas_table: Vec<GasCost> = Vec::new();

        for (token, gas_cost) in gas_cost_table {
            gas_table.push(GasCost {
                token_address: token.to_string(),
                amount: gas_cost.to_string_native(),
            })
        }

        gas_table
    }
}
