#![allow(clippy::tests_outside_test_module)]

use axum::{
	body::Body,
	http::{Request, StatusCode},
	response::Response,
};
use http_body_util::BodyExt;
use mockall::predicate;
// for `collect`
use sqlx::PgPool;
use tower::ServiceExt; // for `call`, `oneshot`, and `ready`
use youtube_summarizer_server::web::{
	routes::routes,
	services::{transcript::get_transcript_by_url, youtube::MockYtServiceTrait},
	utils::YTUrl,
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

	let mut yt_service = MockYtServiceTrait::new();
	yt_service
		.expect_fetch_captions()
		.with(predicate::eq(url.clone()))
		.times(1)
		.returning(|_url| Ok(VTT.to_owned()));

	let transcript = get_transcript_by_url(&url, &pool, yt_service).await?;
	assert_eq!(transcript, CLEAN_VTT);

	// should be stored in DB
	let mut yt_service = MockYtServiceTrait::new();
	yt_service
		.expect_fetch_captions()
		.times(0);

	let transcript = get_transcript_by_url(&url, &pool, yt_service).await?;
	assert_eq!(transcript, CLEAN_VTT);

	Ok(())
}

#[sqlx::test]
async fn invalid_url(pool: PgPool) -> Result<()> {
	let routes = routes(pool);

	let response = routes
		.oneshot(
			Request::builder()
				.uri("/transcript?url=invalid_url")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	let body = body_to_string(response).await?;

	assert_eq!(body, "Invalid URL");

	Ok(())
}
