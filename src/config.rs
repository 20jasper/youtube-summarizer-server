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

		let api_key = env::var("OPEN_AI_API_KEY")?;
		let model = env::var("OPEN_AI_MODEL")?;
		let base_url = env::var("OPEN_AI_BASE_URL")?;

		Ok(Self {
			api_key,
			model,
			base_url,
		})
	}
}
