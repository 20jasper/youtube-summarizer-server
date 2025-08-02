use crate::error::Result;
use crate::prompts::ARTICLE_TEMPLATE;
use crate::web::clients::CompletionClient;
use crate::web::services::youtube::YtService;
use crate::web::utils::YTUrl;
use core::str;
use core::time::Duration;
use regex::Regex;
use sqlx::PgPool;
use std::borrow::Cow;
use tokio::time::timeout;

pub async fn get_transcript_by_url(url: &YTUrl, pool: &PgPool) -> Result<String> {
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
			tokio::spawn(async move {
				let client = YtService::from_env()?;
				client.fetch_captions(&owned_url)
			}),
		)
		.await???;
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

	Ok(clean_vtt(&transcript))
}

pub async fn summarize_by_url(url: &YTUrl, pool: &PgPool) -> Result<String> {
	let summary = CompletionClient::from_env()?
		.post(ARTICLE_TEMPLATE, &get_transcript_by_url(url, pool).await?)
		.await?;

	tracing::debug!("summarized transcript");

	Ok(summary)
}

/// remove timestamps and duplicate lines
pub fn clean_vtt(transcript: &str) -> String {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_convert_vtt_to_text() {
		let vtt = "WEBVTT
Kind: captions
Language: en
00:00:00.580 --> 00:00:01.910 align:start position:0%
[Music]
00:00:01.910 --> 00:00:01.920 align:start position:0%
[Music]

00:00:01.920 --> 00:00:04.150 align:start position:0%
[Music]
you<00:00:02.040><c> know</c><00:00:02.200><c> what's</c><00:00:02.520><c> really</c><00:00:02.840><c> not</c><00:00:03.120><c> fun</c><00:00:03.679><c> recording</c>

00:00:04.150 --> 00:00:04.160 align:start position:0%
you know what's really not fun recording

00:00:04.160 --> 00:00:06.510 align:start position:0%
you know what's really not fun recording
an<00:00:04.359><c> entire</c><00:00:04.880><c> video</c><00:00:05.160><c> for</c><00:00:05.400><c> 30</c><00:00:05.720><c> minutes</c><00:00:06.200><c> and</c><00:00:06.319><c> then</c>

00:00:06.510 --> 00:00:06.520 align:start position:0%
an entire video for 30 minutes and then
 

00:00:06.520 --> 00:00:08.669 align:start position:0%
an entire video for 30 minutes and then
realizing<00:00:07.359><c> you</c><00:00:07.520><c> forgot</c><00:00:07.839><c> to</c><00:00:08.080><c> plug</c><00:00:08.280><c> in</c><00:00:08.440><c> your</c>";

		assert_eq!(
			clean_vtt(vtt),
			"[Music] you know what's really not fun recording an entire video for 30 minutes and then"
		);
	}
}
