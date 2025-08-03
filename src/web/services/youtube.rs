use crate::web::clients::YtdlpClientBuilder;
use crate::web::utils::YTUrl;
use crate::{
	error::{Error, Result},
	web::services::env::load_env,
};
use mockall::automock;
use reqwest::Url;

#[automock]
pub trait YtServiceTrait {
	fn fetch_captions(&self, url: &YTUrl) -> Result<String>;
}

pub struct YtService {
	retries: u8,
	proxy: Url,
}

impl YtService {
	pub fn new(retries: u8, proxy: Url) -> Self {
		Self { retries, proxy }
	}

	pub fn from_env() -> Result<Self> {
		const YOUTUBE_PROXY: &str = "YOUTUBE_PROXY";
		const YOUTUBE_RETRIES: &str = "YOUTUBE_RETRIES";
		load_env()?;

		let proxy = std::env::var(YOUTUBE_PROXY).map_err(|_| Error::EnvMissing(YOUTUBE_PROXY))?;
		let proxy = Url::parse(&proxy)?;
		let retries = std::env::var(YOUTUBE_RETRIES)
			.ok()
			.and_then(|s| s.parse::<u8>().ok())
			.unwrap_or(3);

		Ok(Self::new(retries, proxy))
	}
}

impl YtServiceTrait for YtService {
	fn fetch_captions(&self, url: &YTUrl) -> Result<String> {
		YtdlpClientBuilder::default()
			.proxy(self.proxy.clone())
			.retries(self.retries)
			.download_subtitles(true)
			.build()
			.unwrap()
			.request(url)
	}
}
