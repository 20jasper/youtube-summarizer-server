#![allow(clippy::tests_outside_test_module)]

use axum::{
	body::Body,
	http::{self, Request, StatusCode},
	response::Response,
};
use http_body_util::BodyExt; // for `collect`
use mockall::{mock, predicate};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`
use youtube_summarizer_server::{
	error::ErrorMessage,
	prompts::ONESHOT_SUMMARY_TEMPLATE,
	web::{
		clients::CompletionClient,
		routes::routes,
		services::{
			summary::summarize_by_url, transcript::get_transcript_by_url, youtube::MockYtService,
		},
		utils::YTUrl,
	},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn body_to_string(res: Response) -> Result<String> {
	let bytes = res
		.into_body()
		.collect()
		.await?
		.to_bytes()
		.to_vec();
	Ok(String::from_utf8(bytes)?)
}

#[sqlx::test]
fn can_build_router(pool: PgPool) -> Result<()> {
	let routes = routes(pool);

	let response = routes
		.oneshot(
			Request::builder()
				.uri("/")
				.body(Body::empty())
				.unwrap(),
		)
		.await?;

	assert_eq!(response.status(), StatusCode::OK);

	let body = body_to_string(response).await?;

	assert_eq!(body, "hello world");

	Ok(())
}

const VTT: &str = include_str!("./test.vtt");
const CLEAN_VTT: &str =
	"[Music] you know what's really not fun recording an entire video for 30 minutes and then";

#[sqlx::test]
async fn should_get_and_cache_transcript(pool: PgPool) -> Result<()> {
	let url = YTUrl::try_from(
		"https://www.youtube.com/watch?v=DjcC6p_8fpE&pp=ygUWamFjb2IgYXNwZXIgdHlwZXNjcmlwdA%3D%3D",
	)?;

	let mut yt_service = MockYtService::new();
	yt_service
		.expect_fetch_captions()
		.with(predicate::eq(url.clone()))
		.times(1)
		.returning(|_url| Ok(VTT.to_owned()));

	let transcript = get_transcript_by_url(&url, &pool, yt_service).await?;
	assert_eq!(transcript, CLEAN_VTT);

	// should be stored in DB
	let mut yt_service = MockYtService::new();
	yt_service
		.expect_fetch_captions()
		.times(0);

	let transcript = get_transcript_by_url(&url, &pool, yt_service).await?;
	assert_eq!(transcript, CLEAN_VTT);

	Ok(())
}

mock! {
	pub Completion {}
	#[allow(refining_impl_trait, reason="this is for tests and for some reason it doesn't like impls in returns")]
	impl CompletionClient for Completion {
		fn post(&self, prompt: &str, text: &str) -> impl Future<Output = youtube_summarizer_server::error::Result<String>> + Send;
		fn post_stream(self, prompt: &str, text: &str) -> impl Future<Output = youtube_summarizer_server::error::Result<core::pin::Pin<Box<dyn futures::Stream<Item = axum::response::sse::Event> + Send>>>> + Send;
	}

	impl Clone for Completion {
		fn clone(&self) -> Self;
	}
}
#[sqlx::test]
async fn should_get_and_cache_summary(pool: PgPool) -> Result<()> {
	let url = YTUrl::try_from(
		"https://www.youtube.com/watch?v=DjcC6p_8fpE&pp=ygUWamFjb2IgYXNwZXIgdHlwZXNjcmlwdA%3D%3D",
	)?;
	let summary = "Cheese is scrumptious";

	let mut yt_service = MockYtService::new();
	yt_service
		.expect_fetch_captions()
		.with(predicate::eq(url.clone()))
		.times(1)
		.returning(|_url| Ok(VTT.to_owned()));

	let mut client = MockCompletion::new();
	client
		.expect_post()
		.with(
			predicate::eq(ONESHOT_SUMMARY_TEMPLATE),
			predicate::eq(CLEAN_VTT),
		)
		.times(1)
		.returning(|_prompt, _text| Box::pin(async { Ok(summary.to_owned()) }));

	let res = summarize_by_url(&url, &pool, yt_service, &client).await?;
	assert_eq!(res, summary);

	let mut yt_service = MockYtService::new();
	yt_service
		.expect_fetch_captions()
		.times(0);

	let mut client = MockCompletion::new();
	client.expect_post().times(0);

	let res = summarize_by_url(&url, &pool, yt_service, &client).await?;
	assert_eq!(res, summary);

	Ok(())
}

const TEST_ID: &str = "TEST_ID";
#[sqlx::test(fixtures(path = "fixtures", scripts("video_with_summary")))]
async fn should_submit_feedback_for_existing_summary(pool: PgPool) -> Result<()> {
	let message = "rust is a must";
	let routes = routes(pool.clone());

	let response = routes
		.oneshot(
			Request::builder()
				.uri("/summary/rating".to_string())
				.method(http::Method::POST)
				.header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
				.body(Body::from(
					json!({"rating": "dislike", "message": message, "videoId": TEST_ID})
						.to_string(),
				))?,
		)
		.await?;
	assert_eq!(response.status(), StatusCode::OK);

	let res = sqlx::query!("SELECT message FROM ratings WHERE video_ID = $1", TEST_ID)
		.fetch_one(&pool)
		.await?;
	assert_eq!(res.message, Some(message.into()));

	Ok(())
}

async fn transcript_error(pool: PgPool, url: &str, error_message: &str) -> Result<()> {
	let routes = routes(pool);

	let response = routes
		.oneshot(
			Request::builder()
				.uri(format!("/transcript?url={url}"))
				.body(Body::empty())?,
		)
		.await?;

	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	let body = body_to_string(response).await?;

	let ErrorMessage { error, message } = serde_json::from_str::<ErrorMessage>(&body)?;

	assert!(message.contains(error_message));
	assert!(message.contains(url));
	assert!(error);

	Ok(())
}

#[sqlx::test]
async fn invalid_url(pool: PgPool) -> Result<()> {
	let url = "uhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh";
	let error_message = "Invalid URL";
	transcript_error(pool, url, error_message).await
}

#[sqlx::test]
async fn unsupported_url(pool: PgPool) -> Result<()> {
	let url = "https://www.rustisamust.com/watch";
	let error_message = "Unsupported URL";
	transcript_error(pool, url, error_message).await
}
