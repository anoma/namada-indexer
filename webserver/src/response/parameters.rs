use orm::parameters::ParametersDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub unbonding_length: String,
    pub pipeline_length: String,
    pub epochs_per_year: String,
}

impl From<ParametersDb> for Parameters {
    fn from(parameters: ParametersDb) -> Self {
        Self {
            unbonding_length: parameters.unbonding_length.to_string(),
            pipeline_length: parameters.pipeline_length.to_string(),
            epochs_per_year: parameters.epochs_per_year.to_string(),
        }
    }
}
