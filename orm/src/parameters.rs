use std::str::FromStr;

use bigdecimal::BigDecimal;
use diesel::prelude::Insertable;
use diesel::query_builder::AsChangeset;
use diesel::{Queryable, Selectable};
use serde_json::Value as SerdeJSONValue;
use shared::checksums::Checksums;
use shared::genesis::Genesis;
use shared::parameters::{EpochSwitchBlocksDelay, Parameters};

use crate::schema::chain_parameters;

#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = chain_parameters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ParametersInsertDb {
    pub unbonding_length: i32,
    pub pipeline_length: i32,
    pub epochs_per_year: i32,
    pub min_num_of_blocks: i32,
    pub min_duration: i32,
    pub max_block_time: i32,
    pub apr: String,
    pub native_token_address: String,
    pub chain_id: String,
    pub genesis_time: i64,
    pub checksums: SerdeJSONValue,
    pub epoch_switch_blocks_delay: i32,
    pub cubic_slashing_window_length: i32,
    pub duplicate_vote_min_slash_rate: BigDecimal,
    pub light_client_attack_min_slash_rate: BigDecimal,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = chain_parameters)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ParametersDb {
    pub id: i32,
    pub unbonding_length: i32,
    pub pipeline_length: i32,
    pub epochs_per_year: i32,
    pub min_num_of_blocks: i32,
    pub min_duration: i32,
    pub max_block_time: i32,
    pub apr: String,
    pub native_token_address: String,
    pub chain_id: String,
    pub genesis_time: i64,
    pub checksums: SerdeJSONValue,
    pub epoch_switch_blocks_delay: i32,
    pub cubic_slashing_window_length: i32,
    pub duplicate_vote_min_slash_rate: BigDecimal,
    pub light_client_attack_min_slash_rate: BigDecimal,
}

impl From<(Parameters, Genesis, Checksums, EpochSwitchBlocksDelay)>
    for ParametersInsertDb
{
    fn from(
        (parameters, genesis, checksums, epoch_switch_blocks_delay): (
            Parameters,
            Genesis,
            Checksums,
            EpochSwitchBlocksDelay,
        ),
    ) -> Self {
        Self {
            unbonding_length: parameters.unbonding_length as i32,
            pipeline_length: parameters.pipeline_length as i32,
            epochs_per_year: parameters.epochs_per_year as i32,
            min_num_of_blocks: parameters.min_num_of_blocks as i32,
            min_duration: parameters.min_duration as i32,
            max_block_time: parameters.max_block_time as i32,
            apr: parameters.apr,
            native_token_address: parameters.native_token_address,
            chain_id: genesis.chain_id,
            genesis_time: genesis.genesis_time,
            checksums: serde_json::to_value(checksums)
                .expect("Failed to serialize checksums"),
            epoch_switch_blocks_delay: epoch_switch_blocks_delay as i32,
            cubic_slashing_window_length: parameters
                .cubic_slashing_window_length
                as i32,
            duplicate_vote_min_slash_rate: BigDecimal::from_str(
                &parameters.duplicate_vote_min_slash_rate,
            )
            .expect("Invalid duplicate_vote_min_slash_rate"),
            light_client_attack_min_slash_rate: BigDecimal::from_str(
                &parameters.light_client_attack_min_slash_rate,
            )
            .expect("Invalid light_client_attack_min_slash_rate"),
        }
    }
}

impl From<ParametersDb> for Parameters {
    fn from(parameters: ParametersDb) -> Parameters {
        Parameters {
            unbonding_length: parameters.unbonding_length as u64,
            pipeline_length: parameters.pipeline_length as u64,
            epochs_per_year: parameters.epochs_per_year as u64,
            min_num_of_blocks: parameters.min_num_of_blocks as u64,
            min_duration: parameters.min_duration as u64,
            max_block_time: parameters.max_block_time as u64,
            apr: parameters.apr,
            native_token_address: parameters.native_token_address,
            cubic_slashing_window_length: parameters
                .cubic_slashing_window_length
                as u64,
            duplicate_vote_min_slash_rate: parameters
                .duplicate_vote_min_slash_rate
                .to_string(),
            light_client_attack_min_slash_rate: parameters
                .light_client_attack_min_slash_rate
                .to_string(),
        }
    }
}
