use crate::web::{routes::routes, services::env::ApplicationSettings};
use sqlx::PgPool;
use std::io;
use tokio::net::TcpListener;

pub mod error;
pub mod prompts;
pub mod web;

pub async fn run(
	listener: TcpListener,
	pool: PgPool,
	application: &ApplicationSettings,
) -> io::Result<()> {
	let routes = routes(pool, application);
	axum::serve(listener, routes.into_make_service()).await
}
