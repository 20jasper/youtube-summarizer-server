use crate::{
	error::Result,
	prompts::{CHUNKED_COMBINE_TEMPLATE, CHUNKED_SUMMARY_TEMPLATE, ONESHOT_SUMMARY_TEMPLATE},
	web::{
		clients::CompletionClient,
		services::{transcript::get_transcript_by_url, youtube::YtService},
		utils::YTUrl,
	},
};
use sqlx::PgPool;
use tokio::task::JoinSet;

pub async fn summarize_by_url(
	url: &YTUrl,
	pool: &PgPool,
	yt_service: impl YtService + Send + Sync + 'static,
	client: &(impl CompletionClient + Clone + Send + Sync + 'static),
) -> Result<String> {
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
		let transcript = get_transcript_by_url(url, pool, yt_service).await?;
		tracing::debug!("transcript len: {}", transcript.len());

		let summary = summary(client, &transcript, 10_000, 100).await?;

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

async fn oneshot_summary(client: &impl CompletionClient, transcript: &str) -> Result<String> {
	client
		.post(ONESHOT_SUMMARY_TEMPLATE, transcript)
		.await
}

async fn chunk_summary(client: impl CompletionClient, chunk: String) -> Result<String> {
	client
		.post(CHUNKED_SUMMARY_TEMPLATE, &chunk)
		.await
}

async fn summary(
	client: &(impl CompletionClient + Clone + Send + Sync + 'static),
	transcript: &str,
	size: usize,
	overlap: usize,
) -> Result<String> {
	let chunks = chunk_text_by_words(transcript, size, overlap);
	let len = chunks.len();

	tracing::debug!("{len} chunks");

	if len == 1 {
		tracing::debug!("using oneshot prompt");
		return oneshot_summary(client, transcript).await;
	}
	tracing::debug!("using chunked prompts");

	multi_chunk_summary(client, chunks).await
}

async fn multi_chunk_summary(
	client: &(impl CompletionClient + Clone + Send + Sync + 'static),
	chunks: Vec<String>,
) -> Result<String> {
	let len = chunks.len();

	let summaries = chunks
		.into_iter()
		.map(|x| chunk_summary(client.clone(), x))
		.collect::<JoinSet<_>>()
		.join_all()
		.await
		.into_iter()
		.collect::<Result<Vec<String>>>()?;

	let combined = summaries
		.iter()
		.enumerate()
		.map(|(i, summary)| format!("chunk {}/{}\n{summary}", i + 1, len))
		.collect::<Vec<_>>()
		.join("\n");

	tracing::debug!(combined);

	client
		.post(CHUNKED_COMBINE_TEMPLATE, &combined)
		.await
}

/// overlapping chunks of text by `size` words
/// returns 1 chunk if `size` <= `overlap`
fn chunk_text_by_words(s: &str, size: usize, overlap: usize) -> Vec<String> {
	let offset = if let Some(offset) = size.checked_sub(overlap)
		&& offset > 0
	{
		offset
	} else {
		return vec![s.into()];
	};

	let words = s
		.split_ascii_whitespace()
		.collect::<Vec<_>>();

	if words.len() <= size {
		return vec![s.into()];
	}

	let chunks = words
		.len()
		.checked_div(offset)
		.expect("offset should never be 0");

	(0..chunks)
		.map(|i| i.saturating_mul(offset))
		.map(|start| start..(start.saturating_add(size)))
		.filter_map(|r| Some(words.get(r)?.to_vec().join(" ")))
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::rstest;

	fn convert<'a>(x: impl IntoIterator<Item = &'a str>) -> Vec<String> {
		x.into_iter()
			.map(str::to_string)
			.collect()
	}

	#[rstest]
	#[case("a b c d e", 2, 1, convert(["a b", "b c", "c d", "d e"]))]
	#[case("a b c d e", 3, 1, convert(["a b c", "c d e"]))]
	#[case("a b c d e", 3, 2, convert(["a b c", "b c d", "c d e"]))]
	#[case("a b c d e", 3, 3, convert(["a b c d e"]))]
	#[case("a b c d e", 1, 3, convert(["a b c d e"]))]
	#[case("a b c d e", 1, 0, convert(["a", "b", "c", "d", "e"]))]
	#[case("a b c d e", 10, 0, convert(["a b c d e"]))]
	fn does_range(
		#[case] text: &str,
		#[case] chunk: usize,
		#[case] overlap: usize,
		#[case] expected: Vec<String>,
	) {
		assert_eq!(chunk_text_by_words(text, chunk, overlap), expected);
	}
}
