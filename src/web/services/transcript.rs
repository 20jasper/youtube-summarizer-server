use crate::error::Result;
use crate::prompts::ARTICLE_TEMPLATE;
use crate::web::clients::CompletionClient;
use crate::web::services::youtube::{YtService, YtServiceTrait};
use crate::web::utils::YTUrl;
use core::str;
use core::time::Duration;
use regex::Regex;
use sqlx::PgPool;
use std::borrow::Cow;
use tokio::time::timeout;

pub async fn get_transcript_by_url(
	url: &YTUrl,
	pool: &PgPool,
	yt_service: impl YtServiceTrait + Send + Sync + 'static,
) -> Result<String> {
	let transcript = if let Ok(row) =
		sqlx::query!("SELECT subtitles FROM videos WHERE video_id = $1", url.id())
			.fetch_one(pool)
			.await
	{
		tracing::debug!("found transcript in database");
		row.subtitles
	} else {
		let owned_url = url.to_owned();
		let transcript = timeout(
			Duration::from_secs(30),
			tokio::spawn(async move { yt_service.fetch_captions(&owned_url) }),
		)
		.await???;
		let transcript = clean_vtt(transcript.as_str());
		tracing::debug!("fetched transcript");

		sqlx::query!(
			"INSERT INTO videos (video_id, subtitles) VALUES ($1, $2)",
			url.id(),
			&transcript
		)
		.execute(pool)
		.await?;

		transcript
	};

	Ok(transcript)
}

pub async fn summarize_by_url(url: &YTUrl, pool: &PgPool) -> Result<String> {
	let summary = if let Ok(row) = sqlx::query!(
		r"
			SELECT summary 
			FROM videos 
			WHERE video_id = $1 AND summary IS NOT NULL
		",
		url.id()
	)
	.fetch_one(pool)
	.await
	{
		tracing::debug!("found summary in database");
		row.summary
			.expect("Summary should not be null")
	} else {
		let summary = CompletionClient::from_env()?
			.post(
				ARTICLE_TEMPLATE,
				&get_transcript_by_url(url, pool, YtService::from_env()?).await?,
			)
			.await?;
		tracing::debug!("summarized transcript");

		sqlx::query!(
			"UPDATE videos SET summary = $1 WHERE video_id = $2",
			summary,
			url.id(),
		)
		.execute(pool)
		.await?;

		summary
	};

	Ok(summary)
}

/// remove timestamps and duplicate lines
fn clean_vtt(transcript: &str) -> String {
	let mut lines = transcript.lines();
	// skip header
	lines.find(|l| l.starts_with("Language"));

	let tags = Regex::new("</*c.*>").unwrap();
	let time_stamp = Regex::new(r"\d{2}:\d{2}:\d{2}\.\d{3}").unwrap();
	lines
		.filter(|l| !time_stamp.is_match(l))
		.map(|l| tags.replace_all(l, ""))
		.filter(|l| !l.trim().is_empty())
		.scan(Cow::from(""), |last_text, l| {
			if &l == last_text {
				Some("".into())
			} else {
				last_text.clone_from(&l);
				Some(l)
			}
		})
		.filter(|l| !l.is_empty())
		.collect::<Vec<_>>()
		.join(" ")
}
