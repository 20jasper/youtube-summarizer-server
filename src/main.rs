use core::net::{Ipv4Addr, SocketAddr};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::sync::Arc;
use tokio::net::TcpListener;
use youtube_summarizer_server::{
	init_tracing,
	web::{
		routes::AppState,
		services::{
			env::{DatabaseSettings, FromEnv, Settings},
			youtube::YtDlpService,
		},
	},
};

pub mod error;
pub mod prompts;
pub mod web;

async fn init_db(settings: DatabaseSettings) -> Result<Pool<Postgres>, sqlx::Error> {
	tracing::info!("Connecting to the database...");
	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&settings.connection_string())
		.await?;
	sqlx::migrate!().run(&pool).await?;
	tracing::info!("Connected to the database");

	Ok(pool)
}

#[tokio::main]
async fn main() {
	let settings = Settings::from_env().unwrap();
	init_tracing(&settings.rust, settings.axiom.as_ref());

	let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, settings.application.port));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();

	let pool = init_db(settings.database)
		.await
		.unwrap();

	tracing::info!("Listening on http://{address}");

	youtube_summarizer_server::run(
		listener,
		AppState {
			pool,
			yt_service: Arc::new(YtDlpService::from_env().unwrap()),
		},
		settings.application,
	)
	.await
	.unwrap();
}
