use crate::web::services::{env::ApplicationSettings, youtube::YtService};
use axum::{Router, routing::get};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod summary;

#[derive(Clone)]
pub struct AppState {
	pub pool: PgPool,
	pub yt_service: Arc<dyn YtService + Send + Sync>,
}

pub fn routes(app_state: AppState, app: &ApplicationSettings) -> Router {
	Router::new()
		.route("/", get(|| async { "hello world" }))
		.nest("/summary", summary::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new(app.public_dir.clone()))
		.layer(TraceLayer::new_for_http())
		.with_state(app_state)
}
