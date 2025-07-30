use std::fs;

use youtube_summarizer_server as yss;
use youtube_summarizer_server::web::services::transcript::{clean_vtt, get_by_url};
use yss::config::Config;
use yss::error::Result;
use yss::web::services::{ai::completions::CompletionClient, transcript};

const PROMPT: &str = "
Act as the author and provide a comprehensive detailed article

You must follow the rules:
	{max_words}

- Use this markdown format
```
# Title

## Subheading 1

Paragraph

## Subheading 2

Another paragraph
- point 1
- point 2
- point 3
```
- Article must be in {language}
- Article must start with an h1 (# Title)
- summary should be informative and act as a replacement for the original transcript to the point that the user doesn't have to go back to read the transcript
- Summary should not mention the author or speaker at all should act as your independent writing without referencing the original transcript or speaker.
";

#[tokio::main]
async fn main() -> Result<()> {
	let Config {
		url,
		api_key,
		model,
		base_url,
	} = Config::build().unwrap();

	let transcript = get_by_url(&url).await?;
	let text = clean_vtt(&transcript);

	let client = CompletionClient::build(api_key, &base_url, model)?;
	let res = client.post(PROMPT, &text).await?;

	let write_path = transcript::get_write_path(&url).ok_or("uh oh")?;
	fs::write(&write_path, res)?;

	println!("written to {}", write_path.display());
	Ok(())
}
