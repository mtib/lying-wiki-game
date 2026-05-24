use axum::http::StatusCode;

pub async fn create_room() -> StatusCode { StatusCode::OK }
pub async fn join_room() -> StatusCode { StatusCode::OK }
pub async fn start_game() -> StatusCode { StatusCode::OK }
pub async fn submit_topic() -> StatusCode { StatusCode::OK }
pub async fn start_round() -> StatusCode { StatusCode::OK }
pub async fn submit_guess() -> StatusCode { StatusCode::OK }
