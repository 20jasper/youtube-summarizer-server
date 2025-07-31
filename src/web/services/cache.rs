use crate::web::services::{transcript::TranscriptState, youtube};
use reqwest::Url;
use std::{env, fs, path::PathBuf};

pub fn get_artifact_dir() -> PathBuf {
	env::var("OUTPUT_PATH")
		.unwrap_or_else(|_| "./transcripts".to_string())
		.into()
}

pub fn get_artifact_path(url: &str, extension: &str) -> Option<PathBuf> {
	let mut path = get_artifact_dir().join(youtube::get_video_id(url)?);
	path.set_extension(extension);

	Some(path)
}

// pub fn put(key: &str, value: &str) -> Result<(), std::io::Error> {

// }

const RAW_EXT: &str = "en.vtt";
const CLEAN_EXT: &str = "en.clean";
const SUMMARY_EXT: &str = "md";
pub struct Key {
	pub url: Url,
	pub state: TranscriptState,
}
pub fn get(key: &Key) -> Option<String> {
	let ext = match key.state {
		TranscriptState::Raw => RAW_EXT,
		TranscriptState::Clean => CLEAN_EXT,
		TranscriptState::Summarized => SUMMARY_EXT,
	};
	let path = get_artifact_path(key.url.as_str(), ext)?;

	fs::read_to_string(&path).ok()
}
