use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use axum_extra::extract::Query;
use futures::Stream;
use tokio_stream::StreamExt;

use crate::dto::chain::{
    CirculatingSupply as CirculatingSupplyDto, TokenSupply as TokenSupplyDto,
};
use crate::error::api::ApiError;
use crate::response::chain::{
    CirculatingSupply as CirculatingSupplyRsp, LastProcessedBlock,
    LastProcessedEpoch, Parameters, RpcUrl, Token,
    TokenSupply as TokenSupplyRsp,
};
use crate::state::common::CommonState;

#[derive(serde::Serialize)]
struct ChainStatusEvent {
    pub height: i32,
    pub epoch: i32,
}

pub async fn chain_status(
    State(state): State<CommonState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(
        tokio::time::interval(Duration::from_secs(3)),
    )
    .then(move |_| {
        let state = state.clone();

        async move {
            let height = state
                .chain_service
                .find_last_processed_block()
                .await
                .expect("Failed to get last processed block");

            let epoch = state
                .chain_service
                .find_last_processed_epoch()
                .await
                .expect("Failed to get last processed epoch");

            let event =
                serde_json::to_string(&ChainStatusEvent { height, epoch })
                    .expect("Failed to serialize event");

            Ok(Event::default().data(event))
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub async fn get_parameters(
    _headers: HeaderMap,
    State(state): State<CommonState>,
) -> Result<Json<Parameters>, ApiError> {
    let parameters = state.chain_service.find_latest_parameters().await?;

    Ok(Json(parameters))
}

pub async fn get_rpc_url(State(state): State<CommonState>) -> Json<RpcUrl> {
    Json(RpcUrl {
        url: state.config.tendermint_url,
    })
}

pub async fn get_tokens(
    State(state): State<CommonState>,
) -> Result<Json<Vec<Token>>, ApiError> {
    let tokens = state.chain_service.find_tokens().await?;
    let res = tokens.into_iter().map(Token::from).collect();

    Ok(Json(res))
}

pub async fn get_last_processed_block(
    State(state): State<CommonState>,
) -> Result<Json<LastProcessedBlock>, ApiError> {
    let last_processed_block =
        state.chain_service.find_last_processed_block().await?;

    Ok(Json(LastProcessedBlock {
        block: last_processed_block.to_string(),
    }))
}

pub async fn get_last_processed_epoch(
    State(state): State<CommonState>,
) -> Result<Json<LastProcessedEpoch>, ApiError> {
    let last_processed_block =
        state.chain_service.find_last_processed_epoch().await?;

    Ok(Json(LastProcessedEpoch {
        epoch: last_processed_block.to_string(),
    }))
}

pub async fn get_token_supply(
    Query(query): Query<TokenSupplyDto>,
    State(state): State<CommonState>,
) -> Result<Json<Option<TokenSupplyRsp>>, ApiError> {
    let supply = state
        .chain_service
        .get_token_supply(query.address, query.epoch)
        .await?;
    Ok(Json(supply))
}

pub async fn get_circulating_supply(
    Query(query): Query<CirculatingSupplyDto>,
    State(state): State<CommonState>,
) -> Result<Json<CirculatingSupplyRsp>, ApiError> {
    let supply = state
        .chain_service
        .get_circulating_supply(query.epoch)
        .await?;
    Ok(Json(supply))
}
