use crate::web::{
	routes::routes,
	services::env::{ApplicationSettings, AxiomSettings},
};
use sqlx::PgPool;
use std::io;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

pub mod error;
pub mod prompts;
pub mod web;

pub async fn run(
	listener: TcpListener,
	pool: PgPool,
	application: ApplicationSettings,
) -> io::Result<()> {
	let routes = routes(pool, &application);
	axum::serve(listener, routes.into_make_service()).await
}

#[allow(unused_variables, reason = "used only in axiom feature")]
pub fn init_tracing(settings: Option<&AxiomSettings>) {
	use tracing_subscriber::{Layer as _, layer::SubscriberExt as _, util::SubscriberInitExt as _};

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
