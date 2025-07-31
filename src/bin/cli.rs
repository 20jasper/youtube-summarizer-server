use std::fs;

use youtube_summarizer_server as yss;
use yss::config::Config;
use yss::error::Result;
use yss::web::services::{
	ai::{completions::CompletionClient, prompt::ARTICLE_TEMPLATE},
	transcript,
};

#[tokio::main]
async fn main() -> Result<()> {
	let Config {
		url,
		api_key,
		model,
		base_url,
	} = Config::build().unwrap();

	let transcript = transcript::get_by_url(&url).await?;
	let text = transcript::clean_vtt(&transcript);

	let client = CompletionClient::build(api_key, &base_url, model)?;
	let res = client
		.post(ARTICLE_TEMPLATE, &text)
		.await?;

	let write_path = transcript::get_write_path(&url).ok_or("uh oh")?;
	fs::write(&write_path, res)?;

	println!("written to {}", write_path.display());
	Ok(())
}
