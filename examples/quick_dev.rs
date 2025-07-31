use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:8080")?;

	println!("time to make a req");

	hc.do_get("/").await?.print().await?;

	hc.do_get("/summary?url=https://www.youtube.com/watch?v=dNY4FKXwTsM")
		.await?
		.print()
		.await?;

	Ok(())
}
