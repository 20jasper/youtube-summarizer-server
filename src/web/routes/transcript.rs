use crate::error::{Error, Result};
use crate::web::clients::DeepInfraClient;
use crate::web::services::summary::summarize_by_url_stream;
use crate::web::services::transcript;
use crate::web::services::youtube::YtDlpService;
use crate::web::utils::YTUrl;
use axum::extract::State;
use axum::response::{Sse, sse};
use axum::routing::post;
use axum::{Json, Router, extract::Query, http::StatusCode, routing::get};
use axum_macros::debug_handler;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing::instrument;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
}

#[instrument(skip(pool), fields(url, video_id = %YTUrl::try_from(url.as_str())?.id()))]
async fn transcript(
	Query(TranscriptParams { url }): Query<TranscriptParams>,
	State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>)> {
	let transcript = transcript::get_transcript_by_url(
		&url.as_str().try_into()?,
		&pool,
		YtDlpService::from_env()?,
	)
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
#[instrument(skip(pool), fields(url, video_id = %YTUrl::try_from(url.as_str())?.id()))]
async fn summarize(
	Query(SummaryParams { url }): Query<SummaryParams>,
	State(pool): State<PgPool>,
) -> Result<Sse<impl futures::Stream<Item = std::result::Result<sse::Event, axum::Error>>>> {
	let stream = summarize_by_url_stream(
		&url.as_str().try_into()?,
		&pool,
		YtDlpService::from_env()?,
		DeepInfraClient::from_env()?,
	)
	.await?
	.map(Ok);
	Ok(Sse::new(stream))
}

#[derive(Deserialize, Debug, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "rating")]
#[sqlx(rename_all = "lowercase")]
enum Rating {
	Like,
	Dislike,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Rate {
	video_id: String,
	rating: Rating,
	message: Option<String>,
}
#[instrument(skip(pool))]
#[debug_handler]
async fn rate(
	State(pool): State<PgPool>,
	Json(Rate {
		video_id,
		rating,
		message,
	}): Json<Rate>,
) -> Result<(StatusCode, ())> {
	#[allow(
		clippy::as_conversions,
		reason = "casting rating as rating to make sqlx happy"
	)]
	let res = sqlx::query!(
		"
			INSERT INTO ratings (video_id, rating, message)
			SELECT $1::VARCHAR, $2, $3
			WHERE EXISTS (
				SELECT *
				FROM videos
				WHERE video_id = $1 AND summary IS NOT NULL
			)
		",
		video_id,
		rating as Rating,
		message,
	)
	.execute(&pool)
	.await?;

	if res.rows_affected() == 0 {
		return Err(Error::NotFound);
	}
	Ok((StatusCode::OK, ()))
}

pub fn routes() -> Router<PgPool> {
	Router::new()
		.route("/summary", get(summarize))
		.route("/summary/rating", post(rate))
		.route("/transcript", get(transcript))
}
