use crate::error::Result;
use crate::web::services::transcript;
use axum::extract::State;
use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
}

async fn transcript(
	Query(TranscriptParams { url }): Query<TranscriptParams>,
	State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>)> {
	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"transcript": transcript::get_transcript_by_url(&url.as_str().try_into()?, &pool).await?
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
