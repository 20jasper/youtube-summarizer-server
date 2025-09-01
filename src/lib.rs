use crate::web::routes::routes;
use sqlx::PgPool;
use tokio::net::TcpListener;

pub mod error;
pub mod prompts;
pub mod web;

pub fn run(
	listener: TcpListener,
	pool: PgPool,
) -> axum::serve::Serve<TcpListener, axum::routing::IntoMakeService<axum::Router>, axum::Router> {
	let routes = routes(pool);

	axum::serve(listener, routes.into_make_service())
}
