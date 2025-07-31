use crate::web::services::transcript;
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
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
	Query(TranscriptParams { url, raw }): Query<TranscriptParams>,
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

#[derive(Deserialize)]
struct SummaryParams {
	url: String,
}
#[debug_handler]
async fn summarize(
	Query(SummaryParams { url }): Query<SummaryParams>,
) -> (StatusCode, Json<Value>) {
	match transcript::summarize_by_url(&url).await {
		Ok(transcript) => (
			StatusCode::OK,
			Json(json!(
					{
						"url": url,
						"transcript": transcript
					}
			)),
		),
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
		.route("/summary", get(summarize))
		.route("/transcript", get(transcript))
}
