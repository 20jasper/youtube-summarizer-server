use crate::error::Result;
use crate::{
	prompts::ONESHOT_SUMMARY_TEMPLATE,
	web::clients::{CompletionClient, DeepInfraClient},
};
use axum::{
	Router,
	response::{Sse, sse::Event},
	routing::get,
};
use futures::StreamExt;
use sqlx::PgPool;

pub async fn stream_summary()
-> Result<Sse<impl futures::Stream<Item = std::result::Result<Event, axum::Error>>>> {
	let client = DeepInfraClient::from_env().unwrap();
	let stream = client
		.post_stream(ONESHOT_SUMMARY_TEMPLATE, "hello gamer")
		.await?
		.map(Ok);
	Ok(Sse::new(stream))
}

pub fn routes() -> Router<PgPool> {
	Router::new().route("/stream", get(stream_summary))
}
