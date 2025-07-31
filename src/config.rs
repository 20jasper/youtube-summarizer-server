use crate::error::Error;
use crate::error::Result;
use dotenvy::dotenv;
use std::env;

pub trait Config {
	fn from_env() -> Result<Self>
	where
		Self: Sized;
}

pub struct Completion {
	pub api_key: String,
	pub model: String,
	pub base_url: String,
}

impl Config for Completion {
	fn from_env() -> Result<Self> {
		dotenv()?;

		const OPEN_AI_API_KEY: &str = "OPEN_AI_API_KEY";
		const OPEN_AI_MODEL: &str = "OPEN_AI_MODEL";
		const OPEN_AI_BASE_URL: &str = "OPEN_AI_BASE_URL";

		let api_key = env::var(OPEN_AI_API_KEY).map_err(|_| Error::EnvMissing(OPEN_AI_API_KEY))?;
		let model = env::var(OPEN_AI_MODEL).map_err(|_| Error::EnvMissing(OPEN_AI_MODEL))?;
		let base_url =
			env::var(OPEN_AI_BASE_URL).map_err(|_| Error::EnvMissing(OPEN_AI_BASE_URL))?;

		Ok(Self {
			api_key,
			model,
			base_url,
		})
	}
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
