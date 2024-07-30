use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use axum_macros::debug_handler;
use shared::balance::DenominatedAmount;

use crate::error::api::ApiError;
use crate::response::balance::AddressBalance;
use crate::state::common::CommonState;

#[debug_handler]
pub async fn get_address_balance(
    _headers: HeaderMap,
    Path(address): Path<String>,
    State(state): State<CommonState>,
) -> Result<Json<Vec<AddressBalance>>, ApiError> {
    let balances = state.balance_service.get_address_balances(address).await?;

    let balances_response: Vec<AddressBalance> = balances
        .iter()
        .map(|balance| AddressBalance {
            token_address: balance.token.to_string(),
            // TODO: temporary solution as we only store NAM balances
            balance: DenominatedAmount::native(balance.amount.clone())
                .to_string_precise(),
        })
        .collect();

    Ok(Json(balances_response))
}
