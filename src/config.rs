use crate::error::Result;
use dotenvy::dotenv;
use std::env;

pub struct Completion {
	pub api_key: String,
	pub model: String,
	pub base_url: String,
}

impl Completion {
	pub fn build() -> Result<Self> {
		dotenv()?;

		let api_key = env::var("OPEN_AI_API_KEY").map_err(|_| "OPEN_AI_API_KEY is not set")?;
		let model = env::var("OPEN_AI_MODEL").map_err(|_| "OPEN_AI_MODEL is not set")?;
		let base_url = env::var("OPEN_AI_BASE_URL").map_err(|_| "OPEN_AI_BASE_URL is not set")?;

		Ok(Self {
			api_key,
			model,
			base_url,
		})
	}
}

pub struct Youtube {
	pub retries: u8,
	pub proxy: String,
}

impl Youtube {
	pub fn build() -> Result<Self> {
		dotenv()?;

		let retries = env::var("YOUTUBE_RETRIES")
			.ok()
			.and_then(|s| s.parse::<u8>().ok())
			.unwrap_or(3);
		let proxy = env::var("YOUTUBE_PROXY").map_err(|_| "YOUTUBE_PROXY is not set")?;

		Ok(Self { retries, proxy })
	}
}
