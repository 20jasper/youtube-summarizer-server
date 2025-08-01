use axum::{routing::get, serve, Router};
use core::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{event, Level};
use tracing_subscriber::EnvFilter;
use web::routes::transcript;

pub mod error;
pub mod prompts;
pub mod web;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(
			EnvFilter::try_from_default_env()
				.or_else(|_| {
					EnvFilter::try_new(
						"youtube_summarizer_server=trace,tower_http=debug,reqwest=trace",
					)
				})
				.unwrap(),
		)
		.init();

	let routes = Router::new()
		.route("/", get(|| async { "hello world" }))
		.merge(transcript::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new("public/"))
		.layer(TraceLayer::new_for_http());

	let address = SocketAddr::from(([0, 0, 0, 0], 8080));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();
	event!(Level::INFO, "Listening on http://{address}");

	serve(listener, routes.into_make_service())
		.await
		.unwrap();
}
