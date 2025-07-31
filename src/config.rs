use crate::error::Error;
use crate::error::Result;
use dotenvy::dotenv;
use std::env;

pub trait Config {
	fn from_env() -> Result<Self>
	where
		Self: Sized;
}

const YOUTUBE_PROXY: &str = "YOUTUBE_PROXY";
const YOUTUBE_RETRIES: &str = "YOUTUBE_RETRIES";
pub struct Youtube {
	pub retries: u8,
	pub proxy: String,
}

impl Config for Youtube {
	fn from_env() -> Result<Self> {
		dotenv()?;

		let retries = env::var(YOUTUBE_RETRIES)
			.ok()
			.and_then(|s| s.parse::<u8>().ok())
			.unwrap_or(3);
		let proxy = env::var(YOUTUBE_PROXY).map_err(|_| Error::EnvMissing(YOUTUBE_PROXY))?;

		Ok(Self { retries, proxy })
	}
}
