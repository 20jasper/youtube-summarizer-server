use axum::{routing::get, serve, Router};
use core::net::SocketAddr;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{event, Level};
use tracing_subscriber::EnvFilter;
use web::routes::transcript;

pub mod error;
pub mod prompts;
pub mod web;

fn init_tracing() {
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
}

async fn init_db() -> Result<Pool<Postgres>, sqlx::Error> {
	tracing::info!("Connecting to the database...");
	PgPoolOptions::new()
		.max_connections(5)
		.connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
		.await
}

#[tokio::main]
async fn main() {
	init_tracing();

	let pool = init_db().await.unwrap();

	let row: (i64,) = sqlx::query_as("SELECT $1")
		.bind(150_i64)
		.fetch_one(&pool)
		.await
		.unwrap();

	assert_eq!(row.0, 150);

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
