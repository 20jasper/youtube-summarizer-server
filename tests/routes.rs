use crate::common::Result;
use mockall::predicate;
use reqwest::StatusCode;
use sqlx::PgPool;
use youtube_summarizer_server::error::ErrorMessage;
use youtube_summarizer_server::web::clients::yt_dlp::{VideoMetaData, metadata::VideoInfo};
use youtube_summarizer_server::web::services::{
	metadata::get_metadata_by_url, youtube::MockYtService,
};
use youtube_summarizer_server::web::utils::YTUrl;

mod common;
mod spawn_app;
use spawn_app::{TestApp, spawn_app};

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

#[tokio::test]
#[rstest::rstest]
#[case("uhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh", "Invalid URL")]
#[case("https://www.rustisamust.com/watch", "Unsupported URL")]
async fn summary_url_validation_errors(#[case] url: &str, #[case] expected: &str) -> Result<()> {
	let TestApp { addr, .. } = spawn_app().await;

	let res = reqwest::Client::new()
		.get(format!("{addr}/summary?url={url}"))
		.send()
		.await?;

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);

	let ErrorMessage { error, message } = serde_json::from_str(&res.text().await?)?;
	assert!(error);
	assert!(message.contains(expected));
	assert!(message.contains(url));

	Ok(())
}
