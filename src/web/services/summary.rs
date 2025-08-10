use crate::{
	error::Result,
	prompts::{CHUNKED_COMBINE_TEMPLATE, CHUNKED_SUMMARY_TEMPLATE, ONESHOT_SUMMARY_TEMPLATE},
	web::{
		clients::{CompletionClient, completions::stream::SseMessage},
		services::{transcript::get_transcript_by_url, youtube::YtService},
		utils::YTUrl,
	},
};
use axum::response::sse;
use core::pin::Pin;
use futures::{Stream, StreamExt as _};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::{
	sync::{Mutex, mpsc},
	task::JoinSet,
};
use tokio_stream::wrappers::ReceiverStream;

pub async fn summarize_by_url_stream(
	url: &YTUrl,
	pool: PgPool,
	yt_service: impl YtService + Send + Sync + 'static,
	client: impl CompletionClient + Clone + Send + Sync + 'static,
) -> Result<Pin<Box<dyn Stream<Item = sse::Event> + Send>>> {
	if let Ok(row) = sqlx::query!(
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
		let summary = row
			.summary
			.expect("Summary should not be null");
		let stream = futures::stream::iter(
			summary
				// TODO chunk data to 4-8KiB for better perf
				.chars()
				.collect::<Vec<_>>()
				.into_iter()
				.map(|c| SseMessage::Message(c.into()).into()),
		)
		.chain(futures::stream::once(async { SseMessage::Done.into() }));
		return Ok(Box::pin(stream));
	}
	let transcript = get_transcript_by_url(url, &pool, yt_service).await?;
	tracing::debug!(
		"transcript len: {}",
		transcript
			.split_ascii_whitespace()
			.count()
	);

	let mut summary_stream = summary(client, &transcript, 10_000, 100).await?;

	let (tx, rx) = mpsc::channel::<sse::Event>(100);
	let cache = Arc::new(Mutex::new(String::with_capacity(2000)));

	let id = url.id().to_owned();

	tokio::spawn(async move {
		while let Some(msg) = summary_stream.next().await {
			// TODO is this just client disconnect?
			if let Err(e) = tx.send(msg.clone().into()).await {
				tracing::error!("error sending message {e:?}");
				break;
			}
			match msg {
				SseMessage::Message(s) => {
					cache.lock().await.push_str(s.as_str());
				}
				SseMessage::Done => break,
				SseMessage::Error => {
					// TODO include more info in this
					tracing::error!("error in SSE stream");
				}
			}
		}

		let summary = { cache.lock().await.clone() };
		let res = sqlx::query!(
			"UPDATE videos SET summary = $1 WHERE video_id = $2",
			summary,
			id,
		)
		.execute(&pool)
		.await;
		if let Ok(res) = res
			&& res.rows_affected() == 1
		{
			tracing::info!("saved summary to database");
		} else {
			tracing::error!("failed to save summary to database");
		}
	});

	Ok(Box::pin(ReceiverStream::new(rx)))
}

async fn oneshot_summary_stream(
	client: impl CompletionClient,
	transcript: &str,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	client
		.post_stream(ONESHOT_SUMMARY_TEMPLATE, transcript)
		.await
}

async fn chunk_summary_oneshot(client: impl CompletionClient, chunk: String) -> Result<String> {
	client
		.post(CHUNKED_SUMMARY_TEMPLATE, &chunk)
		.await
}

async fn summary(
	client: impl CompletionClient + Clone + Send + Sync + 'static,
	transcript: &str,
	size: usize,
	overlap: usize,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	let chunks = chunk_text_by_words(transcript, size, overlap);
	let len = chunks.len();

	tracing::debug!("{len} chunks");

	if len == 1 {
		tracing::debug!("using oneshot prompt");
		return oneshot_summary_stream(client.clone(), transcript).await;
	}
	tracing::debug!("using chunked prompts");

	multi_chunk_summary_stream(client, chunks).await
}

async fn multi_chunk_summary_stream(
	client: impl CompletionClient + Clone + Send + Sync + 'static,
	chunks: Vec<String>,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	let len = chunks.len();

	let summaries = chunks
		.into_iter()
		.map(|x| chunk_summary_oneshot(client.clone(), x))
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
		.post_stream(CHUNKED_COMBINE_TEMPLATE, &combined)
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
		.expect("offset should never be 0")
		+ 1;

	tracing::debug!("{} chunks in func", chunks);

	(0..chunks)
		.map(|i| i.saturating_mul(offset))
		.map(|start| {
			start
				..(start
					.saturating_add(size)
					.min(words.len()))
		})
		.scan(false, |done, r| {
			(!*done).then(|| {
				if r.end >= words.len() {
					*done = true;
				}
				r
			})
		})
		.map(|r| {
			words
				.get(r.clone())
				.expect("chunked text should be in range")
				.to_vec()
				.join(" ")
		})
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

	fn gen_n_words(n: usize) -> String {
		"a ".repeat(n)
	}
	fn word_count(s: &str) -> usize {
		s.split_ascii_whitespace().count()
	}

	#[rstest]
	#[case(6910, 6500, 200, vec![6500, 610])]
	#[case(1200, 1500, 200, vec![1200])]
	#[case(6600, 6500, 200, vec![6500, 300])]
	#[case(7000, 3000, 500, vec![3000, 3000, 2000])]
	#[case(12999, 6500, 0, vec![6500, 6499])]
	#[case(14, 5, 2, vec![5, 5, 5, 5])]
	#[case(16, 5, 2, vec![5, 5, 5, 5, 4])]
	#[case(66000, 10_000, 100, vec![10000, 10000, 10000, 10000, 10000, 10000, 6600])]
	#[case(5, 3, 1, vec![3, 3])]
	#[case(5, 2, 1, vec![2, 2, 2, 2])]
	fn should_have_correct_chunk_sizes(
		#[case] input_size: usize,
		#[case] size: usize,
		#[case] overlap: usize,
		#[case] expected_sizes: Vec<usize>,
	) {
		let chunks = chunk_text_by_words(&gen_n_words(input_size), size, overlap);

		assert_eq!(
			chunks
				.iter()
				.map(|x| x.split_ascii_whitespace().count())
				.collect::<Vec<usize>>(),
			expected_sizes
		);
		assert_eq!(chunks.len(), expected_sizes.len());

		let total_overlap = overlap * (expected_sizes.len() - 1);
		let total_size = chunks
			.iter()
			.map(|x| word_count(x))
			.sum::<usize>();
		assert_eq!(total_size, input_size + total_overlap);
	}
}
