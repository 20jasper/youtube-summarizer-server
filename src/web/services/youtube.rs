use std::path::PathBuf;

use crate::error::Result;
use crate::web::clients::YtdlpClientBuilder;
use crate::web::clients::yt_dlp::VideoMetaData;
use crate::web::services::env::{FromEnv, YouTubeSettings};
use crate::web::utils::YTUrl;
use mockall::automock;
use secrecy::SecretString;

#[automock]
pub trait YtService: Send + Sync + 'static {
	fn fetch_metadata(&self, url: &YTUrl) -> Result<VideoMetaData>;
}

pub struct YtDlpService {
	retries: u8,
	proxy: SecretString,
	output_path: PathBuf,
}

impl YtDlpService {
	pub fn new(retries: u8, proxy: SecretString, output_path: PathBuf) -> Self {
		Self {
			retries,
			proxy,
			output_path,
		}
	}

	pub fn from_env() -> Result<Self> {
		let YouTubeSettings {
			proxy,
			retries,
			output_path,
		} = YouTubeSettings::from_env()?;

		Ok(Self::new(retries, proxy, output_path))
	}
}

impl YtService for YtDlpService {
	fn fetch_metadata(&self, url: &YTUrl) -> Result<VideoMetaData> {
		YtdlpClientBuilder::default()
			.proxy(self.proxy.clone())
			.retries(self.retries)
			.output_path(self.output_path.clone())
			.download_subtitles(true)
			.build()
			.unwrap()
			.fetch_metadata(url)
	}
}
