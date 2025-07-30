use crate::error::Result;
use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub url: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Config {
    pub fn build() -> Result<Self> {
        dotenv()?;
        let mut args = env::args();
        args.next();
        let url = args.next().ok_or("missing url argument")?;

        let api_key = env::var("OPEN_AI_API_KEY")?;
        let model = env::var("OPEN_AI_MODEL")?;
        let base_url = env::var("OPEN_AI_BASE_URL")?;

        Ok(Self {
            url,
            api_key,
            model,
            base_url,
        })
    }
}
