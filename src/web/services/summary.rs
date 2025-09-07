use crate::{
	error::Result,
	prompts::PromptContext,
	web::{
		clients::{CompletionClient, completions::stream::SseMessage, yt_dlp::VideoMetaData},
		services::{metadata::get_metadata_by_url, youtube::YtService},
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
	yt_service: &dyn YtService,
	client: impl CompletionClient + Clone + 'static,
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
			chunk_text_by_chars(&summary, 4_000, 0)
				.into_iter()
				.map(|s| SseMessage::Message(s).into()),
		)
		.chain(futures::stream::once(async { SseMessage::Done.into() }));
		return Ok(Box::pin(stream));
	}
	let VideoMetaData { captions, metadata } = get_metadata_by_url(url, &pool, yt_service).await?;
	tracing::debug!(
		"transcript len: {}",
		captions
			.split_ascii_whitespace()
			.count()
	);
	let mut summary_stream = summary(
		client,
		&captions,
		10_000,
		100,
		&PromptContext::new(metadata.title, metadata.chapters),
	)
	.await?;

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
	ctx: &PromptContext,
	transcript: &str,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	client
		.post_stream(
			PromptContext::ONESHOT_SUMMARY,
			&[ctx.metadata(), transcript.to_string()],
		)
		.await
}

async fn chunk_summary_oneshot(
	client: impl CompletionClient,
	ctx: PromptContext,
	chunk: String,
) -> Result<String> {
	client
		.post(PromptContext::CHUNKED_SUMMARY, &[ctx.metadata(), chunk])
		.await
}

async fn summary(
	client: impl CompletionClient + Clone + 'static,
	transcript: &str,
	size: usize,
	overlap: usize,
	ctx: &PromptContext,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	let chunks = chunk_text_by_words(transcript, size, overlap);
	tracing::debug!("{} chunks", chunks.len());

	if chunks.len() == 1 {
		tracing::debug!("using oneshot prompt");
		return oneshot_summary_stream(client, ctx, transcript).await;
	}
	tracing::debug!("using chunked prompts");

	multi_chunk_summary_stream(client, chunks, ctx).await
}

async fn multi_chunk_summary_stream(
	client: impl CompletionClient + Clone + 'static,
	chunks: Vec<String>,
	ctx: &PromptContext,
) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
	let summaries = chunks
		.into_iter()
		.map(|x| chunk_summary_oneshot(client.clone(), ctx.clone(), x))
		.collect::<JoinSet<_>>()
		.join_all()
		.await
		.into_iter()
		.collect::<Result<Vec<String>>>()?;

	let combined = summaries
		.iter()
		.enumerate()
		.map(|(i, summary)| format!("chunk {}/{}\n{summary}", i + 1, summaries.len()))
		.collect::<Vec<_>>()
		.join("\n");

	tracing::debug!(combined);

	client
		.post_stream(PromptContext::CHUNKED_COMBINE, &[ctx.metadata(), combined])
		.await
}

fn chunk_items<T>(xs: &[T], chunk: usize, overlap: usize) -> Vec<&[T]> {
	let step = if let Some(step) = chunk.checked_sub(overlap)
		&& (1..xs.len()).contains(&step)
	{
		step
	} else {
		return vec![xs];
	};

	let chunks = (xs.len() - chunk).div_ceil(step) + 1;
	(0..xs.len())
		.step_by(step)
		.take(chunks)
		.map(|start| start..(start + chunk).min(xs.len()))
		.map(|r| {
			xs.get(r)
				.expect("chunk should be in range")
		})
		.collect()
}

/// overlapping chunks of text by `size` words
/// returns 1 chunk if `size` <= `overlap`
fn chunk_text_by_words(s: &str, size: usize, overlap: usize) -> Vec<String> {
	let words = s
		.split_ascii_whitespace()
		.collect::<Vec<_>>();

	chunk_items(&words, size, overlap)
		.into_iter()
		.map(|chunk| chunk.join(" "))
		.collect()
}

fn chunk_text_by_chars(s: &str, size: usize, overlap: usize) -> Vec<String> {
	let words = s.chars().collect::<Vec<_>>();

	chunk_items(&words, size, overlap)
		.into_iter()
		.map(|chunk| chunk.iter().collect())
		.collect()
}

#[cfg(test)]
mod tests {
	#![allow(clippy::needless_pass_by_value)]
	#![allow(clippy::indexing_slicing)]

	use core::iter;

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

	#[quickcheck_macros::quickcheck]
	fn overlap_should_be_in_total_length(input: Vec<()>, size: usize, overlap: usize) -> bool {
		let chunks = chunk_items(&input, size, overlap);

		let total_len = chunks
			.iter()
			.map(|x| x.len())
			.sum::<usize>();

		total_len == input.len() + overlap * (chunks.len() - 1)
	}

	#[quickcheck_macros::quickcheck]
	fn chunks_should_overlap(input: Vec<bool>, size: usize, overlap: usize) -> bool {
		let chunks = chunk_items(&input, size, overlap);
		if chunks.len() <= 2 {
			return true;
		}

		chunks
			.windows(2)
			.all(|w| w[0][size - overlap..] == w[1][..overlap])
	}

	#[quickcheck_macros::quickcheck]
	fn no_overlap_concat_is_identity(input: Vec<()>, size: usize) -> bool {
		let chunks = chunk_items(&input, size, 0);

		chunks.concat() == input
	}

	#[quickcheck_macros::quickcheck]
	fn fallback_returns_single_chunk_when_step_invalid(
		input: Vec<u8>,
		chunk: usize,
		overlap: usize,
	) -> bool {
		let step = chunk.saturating_sub(overlap);
		if (1..input.len()).contains(&step) {
			return true;
		}

		let chunks = chunk_items(&input, chunk, overlap);
		chunks == [input.as_slice()]
	}

	#[quickcheck_macros::quickcheck]
	fn reconstruction_identity_with_overlap(
		input: Vec<usize>,
		chunk: usize,
		overlap: usize,
	) -> bool {
		let input_clone = input.clone();
		let chunks = chunk_items(&input_clone, chunk, overlap);
		let combined: Vec<_> = iter::once(chunks[0])
			.chain(
				chunks
					.iter()
					.skip(1)
					.map(|c| &c[overlap..]),
			)
			.collect();

		combined.concat() == input
	}

	#[quickcheck_macros::quickcheck]
	fn empty_input_always_single_empty_chunk(chunk: usize, overlap: usize) -> bool {
		let chunks = chunk_items::<usize>(&[], chunk, overlap);
		chunks.len() == 1 && chunks[0].is_empty()
	}
}
