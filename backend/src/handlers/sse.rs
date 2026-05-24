use axum::{extract::{Path, State}, http::StatusCode};
use std::sync::Arc;
use crate::AppState;

pub async fn sse_handler(
    State(_state): State<Arc<AppState>>,
    Path(_code): Path<String>,
) -> StatusCode {
    StatusCode::OK
}
