use crate::web::services::transcript;
use crate::{error::Result, web::utils::YTUrl};
use axum::extract::State;
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
}

async fn transcript(
	Query(TranscriptParams { url }): Query<TranscriptParams>,
	State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>)> {
	let url = YTUrl::parse_from_str(&url)?;

	let transcript = if let Ok(row) = sqlx::query!(
		"SELECT subtitles FROM videos WHERE video_id = $1",
		url.id_string()
	)
	.fetch_one(&pool)
	.await
	{
		tracing::debug!("found transcript in database");
		row.subtitles
	} else {
		let transcript = transcript::get_transcript_by_url(&url, false).await?;

		sqlx::query!(
			"INSERT INTO videos (video_id, subtitles) VALUES ($1, $2)",
			url.id_string(),
			transcript
		)
		.execute(&pool)
		.await?;

		transcript
	};

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
#[debug_handler]
async fn summarize(
	Query(SummaryParams { url }): Query<SummaryParams>,
) -> Result<(StatusCode, Json<Value>)> {
	Ok((
		StatusCode::OK,
		Json(json!(
				{
					"summary":transcript::summarize_by_url(&YTUrl::parse_from_str(&url)?).await?,
				}
		)),
	))
}

pub fn routes() -> Router<PgPool> {
	Router::new()
		.route("/summary", get(summarize))
		.route("/transcript", get(transcript))
}
