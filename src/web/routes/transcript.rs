use crate::web::services::transcript;
use crate::web::services::transcript::clean_vtt;
use axum::{http::StatusCode, routing::post, Json, Router};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
	#[serde(default)]
	raw: bool,
}

#[debug_handler]
async fn transcript(
	Json(TranscriptParams { url, raw }): Json<TranscriptParams>,
) -> (StatusCode, Json<Value>) {
	println!("post transcript: {url:?}, raw {raw:?}");

	if let Ok(transcript) = transcript::get_by_url(&url).await {
		println!("got transcript");
		(
			StatusCode::OK,
			Json(json!(
					{
						"url": url,
						"transcript": if raw {transcript} else {clean_vtt(&transcript)}
					}
			)),
		)
	} else {
		println!("failed to get transcript");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"message": "internal server error"})),
		)
	}
}

pub fn routes() -> Router {
	Router::new().route("/transcript", post(transcript))
}
