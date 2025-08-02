use axum::{Router, routing::get, serve};
use core::net::{Ipv4Addr, SocketAddr};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{Level, event};
use tracing_subscriber::EnvFilter;
use web::routes::transcript;
use youtube_summarizer_server::web::services::env::load_env;

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
	let db = PgPoolOptions::new()
		.max_connections(5)
		.connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
		.await;
	tracing::info!("Connected to the database");
	db
}

#[tokio::main]
async fn main() {
	load_env().unwrap();
	init_tracing();

	let pool = init_db().await.unwrap();
	sqlx::migrate!()
		.run(&pool)
		.await
		.unwrap();

	let routes = Router::new()
		.route("/", get(|| async { "hello world" }))
		.merge(transcript::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new(
			env::var("PUBLIC_DIR").unwrap_or("public/".into()),
		))
		.layer(TraceLayer::new_for_http())
		.with_state(pool);

	let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();
	event!(Level::INFO, "Listening on http://{address}");

	serve(listener, routes.into_make_service())
		.await
		.unwrap();
}
