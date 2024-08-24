use axum::{
	extract::{Path, Query},
	response::Html,
	routing::{get, post},
	Json, Router,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
}

#[debug_handler]
async fn transcript(Json(TranscriptParams { url }): Json<TranscriptParams>) -> Json<Value> {
	Json(json!({ "url": url }))
}

pub fn routes() -> Router {
	Router::new().route("/transcript", post(transcript))
}
