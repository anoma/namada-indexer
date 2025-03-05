use orm::masp::{
    MaspPoolAggregateKindDb, MaspPoolAggregateWindowDb, MaspPoolDb,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaspPoolAggregateWindow {
    OneDay,
    SevenDays,
    ThirtyDays,
    AllTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MaspPoolAggregateKind {
    Inflows,
    Outflows,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaspPoolAggregateResponse {
    pub token_address: String,
    pub time_window: MaspPoolAggregateWindow,
    pub kind: MaspPoolAggregateKind,
    pub total_amount: String,
}

impl From<MaspPoolDb> for MaspPoolAggregateResponse {
    fn from(value: MaspPoolDb) -> Self {
        MaspPoolAggregateResponse {
            token_address: value.token_address,
            time_window: match value.time_window {
                MaspPoolAggregateWindowDb::OneDay => {
                    MaspPoolAggregateWindow::OneDay
                }
                MaspPoolAggregateWindowDb::SevenDays => {
                    MaspPoolAggregateWindow::SevenDays
                }
                MaspPoolAggregateWindowDb::ThirtyDays => {
                    MaspPoolAggregateWindow::ThirtyDays
                }
                MaspPoolAggregateWindowDb::AllTime => {
                    MaspPoolAggregateWindow::AllTime
                }
            },
            kind: match value.kind {
                MaspPoolAggregateKindDb::Inflows => {
                    MaspPoolAggregateKind::Inflows
                }
                MaspPoolAggregateKindDb::Outflows => {
                    MaspPoolAggregateKind::Outflows
                }
            },
            total_amount: value.total_amount.to_string(),
        }
    }
}
