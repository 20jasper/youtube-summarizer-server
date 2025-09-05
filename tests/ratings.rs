mod common;
mod spawn_app;

use common::Result;
use reqwest::StatusCode;
use serde_json::json;
use spawn_app::spawn_app;

use crate::spawn_app::TestApp;

const TEST_ID: &str = "TEST_ID";

#[tokio::test]
async fn should_submit_feedback_for_existing_summary() -> Result<()> {
	let message = "rust is a must";
	let TestApp { addr, pool } = spawn_app().await;

	sqlx::raw_sql(include_str!("fixtures/video_with_summary.sql"))
		.execute(&pool)
		.await?;

	let res = reqwest::Client::new()
		.post(format!("{addr}/summary/rating"))
		.json(&json!({"rating": "dislike", "message": message, "videoId": TEST_ID}))
		.send()
		.await?;

	assert_eq!(res.status(), StatusCode::OK);

	let res = sqlx::query!("SELECT message FROM ratings WHERE video_ID = $1", TEST_ID)
		.fetch_one(&pool)
		.await?;
	assert_eq!(res.message, Some(message.into()));

	Ok(())
}

#[tokio::test]
async fn should_fail_to_rate_for_missing_summary() -> Result<()> {
	let message = "rust is a must";
	let TestApp { addr, .. } = spawn_app().await;

	let res = reqwest::Client::new()
		.post(format!("{addr}/summary/rating"))
		.json(&json!({"rating": "dislike", "message": message, "videoId": TEST_ID}))
		.send()
		.await?;

	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	Ok(())
}
