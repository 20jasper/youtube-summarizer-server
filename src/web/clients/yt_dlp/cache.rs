//! retrieves files saved by the yt-dlp client

use crate::{error::Result, web::utils::YTUrl};
use std::{env, fs, path::PathBuf};

const EXT: &str = "en.vtt";

pub fn get_artifact_dir() -> PathBuf {
	env::var("OUTPUT_PATH")
		.unwrap_or_else(|_| "./transcripts".to_string())
		.into()
}

pub fn get_artifact_path(url: &YTUrl) -> Option<PathBuf> {
	let mut path = get_artifact_dir().join(url.id()?.into_owned());
	path.set_extension(EXT);

	Some(path)
}

pub fn delete(url: &YTUrl) -> Result<()> {
	let path = get_artifact_path(url).ok_or("couldn't compute cache key")?;
	fs::remove_file(&path)?;

	Ok(())
}

pub fn get(url: &YTUrl) -> Option<String> {
	let path = get_artifact_path(url)?;

	fs::read_to_string(&path).ok()
}
