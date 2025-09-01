use crate::web::services::env::ApplicationSettings;
use axum::{Router, routing::get};
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod summary;

pub fn routes(pool: PgPool, app: &ApplicationSettings) -> Router {
	Router::new()
		.route("/", get(|| async { "hello world" }))
		.nest("/summary", summary::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new(app.public_dir.clone()))
		.layer(TraceLayer::new_for_http())
		.with_state(pool)
}
