use crate::web::{routes::routes, services::env::ApplicationSettings};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub mod error;
pub mod prompts;
pub mod web;

pub fn run(
	listener: TcpListener,
	pool: PgPool,
	application: &ApplicationSettings,
) -> axum::serve::Serve<TcpListener, axum::routing::IntoMakeService<axum::Router>, axum::Router> {
	let routes = routes(pool, application);

	axum::serve(listener, routes.into_make_service())
}
