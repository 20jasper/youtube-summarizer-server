use crate::web::services::{env::ApplicationSettings, youtube::YtService};
use axum::{Router, body::Body, http::Request, routing::get};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::Level;

pub mod summary;

#[derive(Clone)]
pub struct AppState {
	pub pool: PgPool,
	pub yt_service: Arc<dyn YtService>,
}

pub fn routes(app_state: AppState, app: &ApplicationSettings) -> Router {
	Router::new()
		.route("/", get(|| async { "hello world" }))
		.nest("/summary", summary::routes())
		// layers run from bottom to top
		.fallback_service(ServeDir::new(app.public_dir.clone()))
		.layer(
			TraceLayer::new_for_http().make_span_with(|req: &Request<Body>| {
				let request_id = uuid::Uuid::new_v4();
				tracing::span!(
					Level::DEBUG,
					"request",
					method = tracing::field::display(req.method()),
					uri = tracing::field::display(req.uri()),
					version = tracing::field::debug(req.version()),
					request_id = tracing::field::display(request_id)
				)
			}),
		)
		.with_state(app_state)
}
