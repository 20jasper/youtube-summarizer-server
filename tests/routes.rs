use crate::common::Result;
use crate::common::body_to_string;
use axum::{
	body::Body,
	http::{Request, StatusCode},
};
use mockall::predicate;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt as _;
use youtube_summarizer_server::{
	error::ErrorMessage,
	web::{
		clients::yt_dlp::{VideoMetaData, metadata::VideoInfo},
		routes::{AppState, routes},
		services::{
			env::ApplicationSettings, metadata::get_metadata_by_url, youtube::MockYtService,
		},
		utils::YTUrl,
	},
};

mod common;

const VTT: &str = include_str!("./test.vtt");
const CLEAN_VTT: &str =
	"[Music] you know what's really not fun recording an entire video for 30 minutes and then";

#[sqlx::test]
async fn should_get_and_cache_metadata(pool: PgPool) -> Result<()> {
	let url = YTUrl::try_from(
		"https://www.youtube.com/watch?v=DjcC6p_8fpE&pp=ygUWamFjb2IgYXNwZXIgdHlwZXNjcmlwdA%3D%3D",
	)?;
	let mock_metadata = VideoInfo {
		title: "Test Video".to_string(),
		duration: Some(300.0),
		description: None,
		tags: vec![],
		thumbnails: vec![],
		chapters: vec![],
		heatmap: vec![],
		channel_id: None,
	};
	let mut yt_service = MockYtService::new();
	let info_clone = mock_metadata.clone();
	yt_service
		.expect_fetch_metadata()
		.with(predicate::eq(url.clone()))
		.times(1)
		.returning(move |_url| {
			Ok(VideoMetaData {
				metadata: info_clone.clone(),
				captions: VTT.to_owned(),
			})
		});

	let VideoMetaData { captions, metadata } =
		get_metadata_by_url(&url, &pool, &yt_service).await?;
	assert_eq!(captions, CLEAN_VTT);
	assert_eq!(metadata, mock_metadata);

	let row = sqlx::query!(
		"SELECT metadata, subtitles, title FROM videos WHERE video_id = $1",
		url.id()
	)
	.fetch_one(&pool)
	.await?;
	let stored: VideoInfo = serde_json::from_value(row.metadata)?;
	assert_eq!(stored, mock_metadata);
	assert_eq!(row.subtitles, CLEAN_VTT.to_string());
	assert_eq!(row.title, mock_metadata.title);

	let mut yt_service = MockYtService::new();
	yt_service
		.expect_fetch_metadata()
		.times(0);

	let VideoMetaData { captions, metadata } =
		get_metadata_by_url(&url, &pool, &yt_service).await?;
	assert_eq!(captions, CLEAN_VTT);
	assert_eq!(metadata, mock_metadata);

	let row = sqlx::query!(
		"SELECT metadata, title FROM videos WHERE video_id = $1",
		url.id()
	)
	.fetch_one(&pool)
	.await?;
	let stored: VideoInfo = serde_json::from_value(row.metadata)?;
	assert_eq!(stored, mock_metadata);
	assert_eq!(row.title, mock_metadata.title);

	Ok(())
}

async fn summary_error(pool: PgPool, url: &str, error_message: &str) -> Result<()> {
	let routes = routes(
		AppState {
			pool,
			yt_service: Arc::new(MockYtService::new()),
		},
		&ApplicationSettings {
			port: 8000,
			public_dir: "doesn't matter".into(),
		},
	);

	let response = routes
		.oneshot(
			Request::builder()
				.uri(format!("/summary?url={url}"))
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
	summary_error(pool, url, error_message).await
}

#[sqlx::test]
async fn unsupported_url(pool: PgPool) -> Result<()> {
	let url = "https://www.rustisamust.com/watch";
	let error_message = "Unsupported URL";
	summary_error(pool, url, error_message).await
}
