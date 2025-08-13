use std::env;

use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod transcript;

pub fn routes(pool: PgPool) -> Router {
	Router::new()
		.route("/", get(|| async { "hello world" }))
		.nest("/summary", transcript::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new(
			env::var("PUBLIC_DIR").unwrap_or("public/".into()),
		))
		.layer(TraceLayer::new_for_http())
		.with_state(pool)
}
