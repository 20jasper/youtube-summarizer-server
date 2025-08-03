use axum::serve;
use core::net::{Ipv4Addr, SocketAddr};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tracing::{Level, event};
use tracing_subscriber::EnvFilter;
use youtube_summarizer_server::web::services::env::load_env;

use crate::web::routes::routes;

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
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
		.await?;
	sqlx::migrate!().run(&pool).await?;
	tracing::info!("Connected to the database");

	Ok(pool)
}

#[tokio::main]
async fn main() {
	load_env().unwrap();
	init_tracing();

	let pool = init_db().await.unwrap();
	let routes = routes(pool);

	let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();
	event!(Level::INFO, "Listening on http://{address}");

	serve(listener, routes.into_make_service())
		.await
		.unwrap();
}
