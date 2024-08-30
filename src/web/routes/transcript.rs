use core::time::Duration;

use axum::{
	http::StatusCode,
	routing::{get, post},
	Json, Router,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{task, time::timeout};

use crate::web::services;
use crate::web::services::transcript;
use crate::web::services::transcript::clean_vtt;

#[derive(Deserialize)]
struct TranscriptParams {
	url: String,
	#[serde(default)]
	raw: bool,
}

#[debug_handler]
async fn transcript(
	Json(TranscriptParams { url, raw }): Json<TranscriptParams>,
) -> (StatusCode, Json<Value>) {
	println!("post transcript: {url:?}, raw {raw:?}");

	if let Ok(transcript) = transcript::get_by_url(&url).await {
		println!("got transcript");
		(
			StatusCode::OK,
			Json(json!(
					{
						"url": url,
						"transcript": if raw {transcript} else {clean_vtt(&transcript)}
					}
			)),
		)
	} else {
		println!("failed to get transcript");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({"message": "internal server error"})),
		)
	}
}

#[debug_handler]
async fn authorize() -> (StatusCode, Json<Value>) {
	println!("authorize");

	let join = task::spawn_blocking(move || services::transcript::authorize().unwrap());

	if (timeout(Duration::from_secs(2), join).await).is_err() {
		println!("server unauthorized");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(
				json!({ "message": "server unauthorized, please contact the server administrator"}),
			),
		)
	} else {
		println!("server authorized");
		(
			StatusCode::OK,
			Json(json!({ "message": "server is authorized"})),
		)
	}
}

pub fn routes() -> Router {
	Router::new()
		.route("/transcript", post(transcript))
		.route("/authorize", get(authorize))
}
