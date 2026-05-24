use axum::http::StatusCode;

pub async fn sse_handler() -> StatusCode { StatusCode::OK }
