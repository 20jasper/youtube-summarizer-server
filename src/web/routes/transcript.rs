use axum::{
	routing::{get, post},
	Json, Router,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::web::services;
use crate::web::services::transcript;
use crate::web::services::transcript::clean_vtt;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
	#[serde(default)]
	raw: bool,
}

#[debug_handler]
async fn transcript(Json(TranscriptParams { url, raw }): Json<TranscriptParams>) -> Json<Value> {
	println!("post transcript: {url:?}, raw {raw:?}");

	let mut transcript = transcript::get_by_url(&url).unwrap();
	if !raw {
		transcript = clean_vtt(&transcript);
	}
	Json(json!({ "url": url, "transcript": transcript }))
}

#[debug_handler]
async fn authorize() -> Json<Value> {
	println!("authorize");

	services::transcript::authorize().unwrap();

	Json(json!({ "message": "please check the server logs"}))
}

pub fn routes() -> Router {
	Router::new()
		.route("/transcript", post(transcript))
		.route("/authorize", get(authorize))
}
