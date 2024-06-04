use orm::revealed_pk::RevealedPkDb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevealedPk {
    pub public_key: Option<String>,
}

impl From<RevealedPkDb> for RevealedPk {
    fn from(value: RevealedPkDb) -> Self {
        Self {
            public_key: Some(value.pk),
        }
    }
}
