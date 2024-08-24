use crate::{Error, Result};

use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use super::AUTH_TOKEN;

pub fn routes() -> Router {
	Router::new().route("/api/login", post(login))
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
	username: String,
	password: String,
}

async fn login(
	cookies: Cookies,
	Json(LoginPayload { username, password }): Json<LoginPayload>,
) -> Result<Json<Value>> {
	// TODO: real auth and db logic
	if username != "jim" || password != "123" {
		return Err(Error::LoginFail);
	}

	// TODO: actually do auth token generation
	cookies.add(Cookie::new(AUTH_TOKEN, "user.expression_date.signature"));

	let body = Json(json!({
		"result": {"success": true}
	}));

	Ok(body)
}
