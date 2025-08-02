//! retrieves files saved by the yt-dlp client

use crate::{
	error::{Error, Result},
	web::utils::YTUrl,
};
use std::{env, fs, path::PathBuf};

const EXT: &str = "en.vtt";

pub fn get_artifact_dir() -> PathBuf {
	env::var("OUTPUT_PATH")
		.unwrap_or_else(|_| "./transcripts".to_string())
		.into()
}

pub fn get_artifact_path(url: &YTUrl) -> std::path::PathBuf {
	let mut path = get_artifact_dir().join(url.id());
	path.set_extension(EXT);
	path
}

/// retrieves and removes the value
pub fn extract(url: &YTUrl) -> Result<String> {
	let path = get_artifact_path(url);

	let val = fs::read_to_string(&path).map_err(|_| Error::CaptionsUnavailable(url.clone()));

	// delete to not waste space
	if let Err(e) = fs::remove_file(&path) {
		tracing::error!("failed to delete file {path}: {e}", path = path.display());
	}

	val
}
