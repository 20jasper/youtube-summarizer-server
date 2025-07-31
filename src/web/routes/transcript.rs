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

#[debug_handler]
async fn summarize(
	Json(TranscriptParams { url, raw }): Json<TranscriptParams>,
) -> (StatusCode, Json<Value>) {
	println!("post transcript: {url:?}, raw {raw:?}");

	match transcript::get_by_url(&url).await {
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
	Router::new().route("/summary", post(summarize))
}
