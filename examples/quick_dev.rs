use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:6981")?;

	hc.do_get("/").await?.print().await?;

	hc.do_get("/authorize")
		.await?
		.print()
		.await?;

	hc.do_post(
		"/transcript",
		json!({"url": "https://www.youtube.com/watch?v=RCsi-w9YbW8"}),
	)
	.await?
	.print()
	.await?;

	Ok(())
}
