use crate::web::services::transcript;
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};
use url::Url;

use crate::error::Result;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
	#[serde(default)]
	raw: bool,
}

async fn transcript(
	Query(TranscriptParams { url, raw }): Query<TranscriptParams>,
) -> Result<(StatusCode, Json<Value>)> {
	println!("GET transcript {url:?}, raw {raw:?}");

	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"url": url,
					"transcript":transcript::get_transcript_by_url(&Url::parse(&url)?, raw).await?
				}
		)),
	))
}

#[derive(Deserialize)]
struct SummaryParams {
	url: String,
}
#[debug_handler]
async fn summarize(
	Query(SummaryParams { url }): Query<SummaryParams>,
) -> Result<(StatusCode, Json<Value>)> {
	println!("GET summary {url:?}");

	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"summary":transcript::summarize_by_url(&Url::parse(&url)?).await?,
				}
		)),
	))
}

pub fn routes() -> Router {
	Router::new()
		.route("/summary", get(summarize))
		.route("/transcript", get(transcript))
}
