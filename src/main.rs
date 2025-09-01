use core::net::{Ipv4Addr, SocketAddr};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer};
use youtube_summarizer_server::web::services::env::{
	AxiomSettings, DatabaseSettings, FromEnv, Settings,
};

pub mod error;
pub mod prompts;
pub mod web;

#[allow(unused_variables, reason = "used only in axiom feature")]
fn init_tracing(settings: Option<&AxiomSettings>) {
	use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

	let fmt_layer = tracing_subscriber::fmt::layer().with_filter(
		EnvFilter::try_from_default_env()
			.or_else(|_| {
				EnvFilter::try_new("youtube_summarizer_server=trace,tower_http=debug,reqwest=trace")
			})
			.unwrap(),
	);
	let registry = tracing_subscriber::registry().with(fmt_layer);

	#[cfg(feature = "axiom")]
	let registry = registry.with(tracing_axiom::default(&settings.unwrap().dataset).unwrap());

	registry.try_init().unwrap();
}

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
	init_tracing(settings.axiom.as_ref());

	let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, settings.application.port));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();

	let pool = init_db(settings.database)
		.await
		.unwrap();

	tracing::info!("Listening on http://{address}");

	youtube_summarizer_server::run(listener, pool, &settings.application)
		.await
		.unwrap();
}
