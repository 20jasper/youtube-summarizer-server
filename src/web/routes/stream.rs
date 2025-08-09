use crate::{
	prompts::ONESHOT_SUMMARY_TEMPLATE,
	web::clients::{
		CompletionClient, DeepInfraClient,
		completions::stream::{SseMessage, SseState},
	},
};
use axum::{
	Router,
	response::{Sse, sse::Event},
	routing::get,
};
use futures::StreamExt;
use sqlx::PgPool;

pub async fn do_thing() -> Sse<impl futures::Stream<Item = Result<Event, axum::Error>>> {
	let client = DeepInfraClient::from_env().unwrap();
	let stream = client
		.post_stream(ONESHOT_SUMMARY_TEMPLATE, "hello gamer")
		.await
		.chain(futures::stream::iter((0..10).map(|_| {
			Event::default()
				.json_data(SseMessage {
					message: None,
					kind: SseState::Done,
				})
				.expect("should always be valid json")
		})))
		.map(Ok);
	Sse::new(stream)
}

pub fn routes() -> Router<PgPool> {
	Router::new().route("/stream", get(do_thing))
}
