use std::{env, path::PathBuf};

use crate::web::services::youtube;

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
