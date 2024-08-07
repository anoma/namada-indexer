#[derive(Debug, Clone)]
pub struct Parameters {
    pub unbonding_length: u64,
    pub pipeline_length: u64,
    pub epochs_per_year: u64,
    pub min_num_of_blocks: u64,
    pub max_block_time: u64,
    pub min_duration: u64,
    pub apr: String,
    pub native_token_address: String,
}

pub type EpochSwitchBlocksDelay = u32;
