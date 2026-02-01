use crate::web::{
	routes::{AppState, routes},
	services::env::{ApplicationSettings, AxiomSettings, RustSettings},
};
use std::io;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

pub mod error;
pub mod prompts;
pub mod web;

pub async fn run(
	listener: TcpListener,
	app_state: AppState,
	application: ApplicationSettings,
) -> io::Result<()> {
	axum::serve(
		listener,
		routes(app_state, &application).into_make_service(),
	)
	.await
}

#[allow(unused_variables, reason = "used only in axiom feature")]
pub fn init_tracing(rust: &RustSettings, axiom: Option<&AxiomSettings>) {
	use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

	let env_filter = EnvFilter::try_new(rust.log.as_str()).unwrap();
	let fmt_layer = tracing_subscriber::fmt::layer();

	let registry = tracing_subscriber::registry()
		.with(env_filter)
		.with(fmt_layer);

	#[cfg(feature = "axiom")]
	{
		let axiom_layer = axiom.map(|AxiomSettings { dataset, .. }| {
			tracing_axiom::default(dataset).expect("failed to init axiom layer")
		});
		if axiom_layer.is_none() {
			tracing::info!("axiom variables not set, not initing axiom");
		}
		registry
			.with(axiom_layer)
			.try_init()
			.unwrap();
	}

	#[cfg(not(feature = "axiom"))]
	registry.try_init().unwrap();
}
