use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use crate::AppState;

#[derive(Serialize)]
pub struct WikiArticle {
    pub title: String,
    pub url: String,
    pub extract: String,
    pub html: String,
}

pub async fn random_article(
    State(state): State<Arc<AppState>>,
) -> Result<Json<WikiArticle>, StatusCode> {
    // Fetch random article summary
    let summary_url = "https://en.wikipedia.org/api/rest_v1/page/random/summary";
    let summary: Value = state
        .http
        .get(summary_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let title = summary["title"].as_str().unwrap_or("").to_string();
    let extract = summary["extract"].as_str().unwrap_or("").to_string();
    let url = summary["content_urls"]["desktop"]["page"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Fetch full mobile-optimised HTML
    let html_url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/mobile-html/{}",
        urlencoding::encode(&title)
    );
    let html = state
        .http
        .get(&html_url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .text()
        .await
        .unwrap_or_default();

    Ok(Json(WikiArticle { title, url, extract, html }))
}
