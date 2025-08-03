use crate::error::Result;
use crate::web::services::transcript;
use crate::web::services::youtube::YtService;
use crate::web::utils::YTUrl;
use axum::extract::State;
use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::instrument;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
}

#[instrument(skip(pool), fields(url, video_id = ?YTUrl::try_from(url.as_str())?.id()))]
async fn transcript(
	Query(TranscriptParams { url }): Query<TranscriptParams>,
	State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>)> {
	let transcript =
		transcript::get_transcript_by_url(&url.as_str().try_into()?, &pool, YtService::from_env()?)
			.await?;
	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"transcript": transcript
				}
		)),
	))
}

#[derive(Deserialize)]
struct SummaryParams {
	url: String,
}
#[instrument(skip(pool), fields(url, video_id = ?YTUrl::try_from(url.as_str())?.id()))]
async fn summarize(
	Query(SummaryParams { url }): Query<SummaryParams>,
	State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>)> {
	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"summary": transcript::summarize_by_url(&url.as_str().try_into()?, &pool).await?,
				}
		)),
	))
}

pub fn routes() -> Router<PgPool> {
	Router::new()
		.route("/summary", get(summarize))
		.route("/transcript", get(transcript))
}
