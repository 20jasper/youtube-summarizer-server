use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:8080")?;

	hc.do_post("/transcript", json!({"url": "youtube.com"}))
		.await?
		.print()
		.await?;

	Ok(())
}
