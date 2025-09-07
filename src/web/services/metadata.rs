use crate::{
	error::{Error, Result},
	web::{clients::yt_dlp::VideoMetaData, services::youtube::YtService, utils::YTUrl},
};
use core::time::Duration;
use regex::Regex;
use sqlx::PgPool;
use std::borrow::Cow;
use tokio::time::timeout;

pub async fn get_metadata_by_url(
	url: &YTUrl,
	pool: &PgPool,
	yt_service: &dyn YtService,
) -> Result<VideoMetaData> {
	let data = if let Ok(row) = sqlx::query!(
		"SELECT subtitles, metadata FROM videos WHERE video_id = $1",
		url.id()
	)
	.fetch_one(pool)
	.await
	{
		tracing::debug!("found transcript in database");
		let transcript = row.subtitles;
		let metadata = serde_json::from_value(row.metadata)
			.map_err(|_| Error::MalformedOrMissingYtMetadata(url.clone()))?;
		VideoMetaData {
			captions: transcript,
			metadata,
		}
	} else {
		let owned_url = url.to_owned();
		let VideoMetaData { metadata, captions } = timeout(Duration::from_secs(30), async move {
			yt_service.fetch_metadata(&owned_url)
		})
		.await??;
		let transcript = clean_vtt(captions.clone().as_str());
		tracing::debug!("fetched transcript");

		let metadata_json: serde_json::Value = serde_json::to_value(&metadata)
			.map_err(|_| Error::MalformedOrMissingYtMetadata(url.clone()))?;

		sqlx::query!(
			"INSERT INTO videos (video_id, subtitles, metadata, title) VALUES ($1, $2, $3, $4)",
			url.id(),
			&transcript,
			metadata_json,
			metadata.title
		)
		.execute(pool)
		.await?;

		VideoMetaData {
			captions: transcript,
			metadata,
		}
	};

	Ok(data)
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
