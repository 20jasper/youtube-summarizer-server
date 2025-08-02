use crate::web::services::transcript;
use crate::{error::Result, web::utils::YTUrl};
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
					"transcript": transcript::get_transcript_by_url(&YTUrl::parse_from_str(&url)?, &pool).await?
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
	let url = YTUrl::parse_from_str(&url)?;

	let summary = if let Ok(row) = sqlx::query!(
		r"
			SELECT summary 
			FROM videos 
			WHERE video_id = $1 AND summary IS NOT NULL
		",
		url.id()
	)
	.fetch_one(&pool)
	.await
	{
		tracing::debug!("found summary in database");
		row.summary
			.expect("Summary should not be null")
	} else {
		let summary = transcript::summarize_by_url(&url, &pool).await?;

		sqlx::query!(
			"UPDATE videos SET summary = $1 WHERE video_id = $2",
			summary,
			url.id(),
		)
		.execute(&pool)
		.await?;

		summary
	};
	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"summary": summary,
				}
		)),
	))
}

pub fn routes() -> Router<PgPool> {
	Router::new()
		.route("/summary", get(summarize))
		.route("/transcript", get(transcript))
}
