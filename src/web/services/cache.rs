use crate::error::Result;
use crate::web::services::transcript::TranscriptState;
use crate::web::services::youtube::YTUrl;
use std::{env, fs, path::PathBuf};

pub fn get_artifact_dir() -> PathBuf {
	env::var("OUTPUT_PATH")
		.unwrap_or_else(|_| "./transcripts".to_string())
		.into()
}

pub fn get_artifact_path(url: &YTUrl, extension: &str) -> Option<PathBuf> {
	let mut path = get_artifact_dir().join(url.id()?.into_owned());
	path.set_extension(extension);

	Some(path)
}

fn state_extension(state: TranscriptState) -> &'static str {
	match state {
		TranscriptState::Raw => "en.vtt",
		TranscriptState::Clean => "en.clean",
		TranscriptState::Summarized => "md",
	}
}

pub fn put(key: &Key, value: &str) -> Result<()> {
	let path = get_artifact_path(&key.url, state_extension(key.state))
		.ok_or("couldn't compute cache key")?;
	fs::write(&path, value)?;

	Ok(())
}

pub struct Key {
	pub url: YTUrl,
	pub state: TranscriptState,
}
pub fn get(key: &Key) -> Option<String> {
	let path = get_artifact_path(&key.url, state_extension(key.state))?;

	fs::read_to_string(&path).ok()
}
