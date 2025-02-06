use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ApiSuccessResponse<T: Serialize> {
    data: T,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ApiErrorResponse {
    message: Option<String>,
    #[serde(rename = "code")]
    status: u16,
}

impl<T: Serialize> ApiSuccessResponse<T>
where
    T: Serialize,
{
    #[allow(dead_code)]
    pub(crate) fn send(data: T) -> Self {
        ApiSuccessResponse { data }
    }
}

impl ApiErrorResponse {
    pub(crate) fn send(status: u16, message: Option<String>) -> Response {
        ApiErrorResponse { message, status }.into_response()
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::from_u16(self.status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(self),
        )
            .into_response()
    }
}
