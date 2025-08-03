#![allow(clippy::tests_outside_test_module)]

use axum::{
	body::Body,
	http::{Request, StatusCode},
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

#[sqlx::test]
fn can_build_router(pool: PgPool) {
	let routes = routes(pool);

	let response = routes
		.oneshot(
			Request::builder()
				.uri("/")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = response
		.into_body()
		.collect()
		.await
		.unwrap()
		.to_bytes();
	assert_eq!(&body[..], b"hello world");
}

const VTT: &str =  "WEBVTT
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
