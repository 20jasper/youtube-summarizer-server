#![allow(clippy::tests_outside_test_module)]

use http_body_util::BodyExt as _;

pub async fn body_to_string(
	res: axum::response::Response,
) -> Result<String, Box<dyn std::error::Error>> {
	let bytes = res
		.into_body()
		.collect()
		.await?
		.to_bytes()
		.to_vec();
	Ok(String::from_utf8(bytes)?)
}
