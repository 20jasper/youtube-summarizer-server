//! retrieves files saved by the yt-dlp client

use crate::{
	error::{Error, Result},
	web::utils::YTUrl,
};
use std::{
	env, fs,
	path::{Path, PathBuf},
};

const CAPTIONS_EXT: &str = "en.vtt";
const METADATA_EXT: &str = "info.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Artifact {
	Captions,
	Metadata,
}

impl Artifact {
	pub fn extension(self) -> &'static str {
		match self {
			Artifact::Captions => CAPTIONS_EXT,
			Artifact::Metadata => METADATA_EXT,
		}
	}

	pub fn dir_from_env() -> PathBuf {
		env::var("OUTPUT_PATH")
			.unwrap_or_else(|_| "./transcripts".to_string())
			.into()
	}

	pub fn path_from_env(self, url: &YTUrl) -> PathBuf {
		self.path(url, &Self::dir_from_env())
	}

	pub fn path(self, url: &YTUrl, dir: &Path) -> PathBuf {
		let mut path = dir.join(url.id());
		path.set_extension(self.extension());
		path
	}

	/// deletes and returns the artifact
	pub fn extract(self, url: &YTUrl) -> Result<String> {
		let path = self.path_from_env(url);
		let val = fs::read_to_string(&path).map_err(|_| Error::CaptionsUnavailable(url.clone()));

		if let Err(e) = fs::remove_file(&path) {
			tracing::error!("failed to delete file {path}: {e}", path = path.display());
		}

		val
	}
}
