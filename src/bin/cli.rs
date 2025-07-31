use std::env;

use url::Url;
use youtube_summarizer_server as yss;
use yss::error::Result;
use yss::web::services::transcript;

#[tokio::main]
async fn main() -> Result<()> {
	let mut args = env::args();
	args.next();
	let url = args
		.next()
		.ok_or("missing url argument")?;
	let url = Url::parse(&url)?;

	transcript::summarize_by_url(&url).await?;

	Ok(())
}
