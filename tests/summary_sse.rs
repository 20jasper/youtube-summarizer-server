#![allow(clippy::tests_outside_test_module)]
use axum::response::sse;
use core::pin::Pin;
use futures::StreamExt as _;
use mockall::mock;
use sqlx::PgPool;
use youtube_summarizer_server::web::clients::CompletionClient;
use youtube_summarizer_server::web::clients::completions::stream::SseMessage;
use youtube_summarizer_server::web::services::summary::summarize_by_url_stream;
use youtube_summarizer_server::web::services::youtube::MockYtService;
use youtube_summarizer_server::web::utils::YTUrl;

mock! {
	pub Completion {}
	#[allow(refining_impl_trait, reason="this is for tests and for some reason it doesn't like impls in returns")]
	impl CompletionClient for Completion {
		fn post(&self, prompt: &str, text: &str) -> impl core::future::Future<Output = youtube_summarizer_server::error::Result<String>> + Send;
		fn post_stream(self, prompt: &str, text: &str) -> impl core::future::Future<Output = youtube_summarizer_server::error::Result<core::pin::Pin<Box<dyn futures::Stream<Item = youtube_summarizer_server::web::clients::completions::stream::SseMessage> + Send>>>> + Send;
	}

	impl Clone for Completion {
		fn clone(&self) -> Self;
	}
}

const TEST_ID: &str = "TEST_ID";
const TRANSCRIPT_NO_SUMMARY: &str = "TEST_ID_NOSUM";

const SUMMARY: &str = "# Gaming Summary\\n\\nGaming time";

#[sqlx::test(fixtures(path = "fixtures", scripts("video_with_summary")))]
async fn should_stream_cached_summary_and_end(
	pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
	let client = MockCompletion::new();
	let yt = MockYtService::new();

	let url: YTUrl = format!("https://www.youtube.com/watch?v={TEST_ID}")
		.as_str()
		.try_into()?;

	let _stream = summarize_by_url_stream(&url, pool.clone(), yt, client).await?;

	let row = sqlx::query!("SELECT summary FROM videos WHERE video_id = $1", TEST_ID)
		.fetch_one(&pool)
		.await?;
	let summary = row
		.summary
		.ok_or("fixture should have summary")?;

	assert_eq!(summary, SUMMARY);

	Ok(())
}

#[sqlx::test(fixtures(path = "fixtures", scripts("video_with_transcript")))]
async fn should_call_client_once_then_cache(
	pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
	let texts = ["Rust ", "Is ", "A ", "Must ", "🦀🦀🦀"];
	let ms = texts
		.into_iter()
		.map(|text| SseMessage::Message(text.into()))
		.chain(std::iter::once(SseMessage::Done))
		.collect::<Vec<_>>();
	let ms_clone = ms.clone();
	let url: YTUrl = format!("https://www.youtube.com/watch?v={TRANSCRIPT_NO_SUMMARY}")
		.as_str()
		.try_into()?;

	let mut client = MockCompletion::new();
	let mut child = MockCompletion::new();
	child
		.expect_post_stream()
		.times(1)
		.returning(move |_, _| {
			Box::pin({
				let ms = ms.clone();
				async move {
					let s: Pin<Box<dyn futures::Stream<Item = SseMessage> + Send>> =
						Box::pin(futures::stream::iter(ms));
					Ok(s)
				}
			})
		});
	client
		.expect_clone()
		.times(1)
		.return_once(move || child);

	let stream = summarize_by_url_stream(&url, pool.clone(), MockYtService::new(), client).await?;
	let events: Vec<sse::Event> = stream.collect().await;
	assert_eq!(events.len(), ms_clone.len());

	let mut client = MockCompletion::new();
	client.expect_post_stream().times(0);

	let _stream = summarize_by_url_stream(&url, pool.clone(), MockYtService::new(), client).await?;

	let row = sqlx::query!(
		"SELECT summary FROM videos WHERE video_id = $1",
		TRANSCRIPT_NO_SUMMARY
	)
	.fetch_one(&pool)
	.await?;
	let summary = row
		.summary
		.ok_or("fixture should have summary")?;

	assert_eq!(summary, texts.join(""));

	Ok(())
}
