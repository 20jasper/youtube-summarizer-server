use crate::web::services::transcript;
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

async fn transcript(
	Json(TranscriptParams { url, raw }): Json<TranscriptParams>,
) -> (StatusCode, Json<Value>) {
	println!("GET transcript {url:?}, raw {raw:?}");

	match transcript::get_transcript_by_url(&url, raw).await {
		Ok(transcript) => {
			println!("got transcript");
			(
				StatusCode::OK,
				Json(json!(
						{
							"url": url,
							"transcript": transcript
						}
				)),
			)
		}
		Err(e) => {
			println!("failed to get transcript {e:?}");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"message": "internal server error"})),
			)
		}
	}
}

#[debug_handler]
async fn summarize(
	Json(TranscriptParams { url, raw }): Json<TranscriptParams>,
) -> (StatusCode, Json<Value>) {
	println!("post transcript: {url:?}, raw {raw:?}");

	match transcript::summarize_by_url(&url).await {
		Ok(transcript) => {
			println!("got transcript");
			(
				StatusCode::OK,
				Json(json!(
						{
							"url": url,
							"transcript": transcript
						}
				)),
			)
		}
		Err(e) => {
			println!("failed to get transcript {e:?}");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({"message": "internal server error"})),
			)
		}
	}
}

pub fn routes() -> Router {
	// todo make these get requests
	Router::new()
		.route("/summary", post(summarize))
		.route("/transcript", post(transcript))
}
