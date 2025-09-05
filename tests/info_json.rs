use youtube_summarizer_server::web::clients::yt_dlp::metadata::VideoInfo;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn deserialize_info_json_fixture() -> TestResult {
	const TITLE: &str = "TypeScript in 100 Seconds";
	const CHANNEL_ID: &str = "UCsBjURrPoezykLs9EqgamOA";
	const JSON: &str = include_str!("./fixtures/zQnBQ4tB3ZA.info.json");
	let parsed: VideoInfo = serde_json::from_str(JSON)?;

	assert_eq!(parsed.title.as_str(), TITLE);
	assert_eq!(parsed.duration, Some(145.0));
	assert_eq!(parsed.channel_id.as_deref(), Some(CHANNEL_ID));

	assert!(!parsed.thumbnails.is_empty());
	assert!(!parsed.tags.is_empty());
	assert!(!parsed.chapters.is_empty());
	assert!(!parsed.heatmap.is_empty());

	Ok(())
}

#[test]
fn empty_strings_coerce_to_none() -> TestResult {
	let title = "Some Title";
	let channel_id = "  ";
	let value = serde_json::json!({
		"title": title,
		"description": "",
		"channel_id": channel_id,
		"duration": null,
		"tags": [],
		"thumbnails": []
	});
	let parsed: VideoInfo = serde_json::from_value(value)?;
	assert_eq!(parsed.title, title);
	assert!(parsed.description.is_none());
	assert!(parsed.channel_id.is_none());

	Ok(())
}
