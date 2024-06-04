#[derive(Debug, Clone)]
pub struct Parameters {
    pub epoch: u32,
    pub unbonding_length: u64,
    pub pipeline_length: u64,
    pub epochs_per_year: u64,
}
