//! retrieves files saved by the yt-dlp client

use crate::{
	error::{Error, Result},
	web::utils::YTUrl,
};
use std::{
	fs,
	path::{Path, PathBuf},
};

const CAPTIONS_EXT: &str = "en.vtt";
const METADATA_EXT: &str = "info.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Artifact {
	Captions(PathBuf),
	Metadata(PathBuf),
}

impl Artifact {
	pub fn extension(&self) -> &'static str {
		match self {
			Artifact::Captions(_) => CAPTIONS_EXT,
			Artifact::Metadata(_) => METADATA_EXT,
		}
	}

	fn dir(&self) -> &Path {
		match self {
			Artifact::Captions(d) | Artifact::Metadata(d) => d,
		}
	}

	pub fn path(self, url: &YTUrl) -> PathBuf {
		let mut path = self.dir().join(url.id());
		path.set_extension(self.extension());
		path
	}

	/// deletes and returns the artifact
	pub fn extract(self, url: &YTUrl) -> Result<String> {
		let path = self.path(url);
		let val = fs::read_to_string(&path).map_err(|_| Error::CaptionsUnavailable(url.clone()));

		if let Err(e) = fs::remove_file(&path) {
			tracing::error!("failed to delete file {path}: {e}", path = path.display());
		}

		val
	}
}
