#[cfg(not(feature = "dotenv"))]
use core::convert::Infallible;

use crate::web::utils::{YTUrl, yt_url};
use axum::{Json, response::IntoResponse};
use core::fmt::{self, Display, Formatter};
use derive_more::From;
use reqwest::StatusCode;
use tokio::{task::JoinError, time::error::Elapsed};
use tracing::{Level, event};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	EnvMissingOrInvalid(&'static str),

	#[cfg(feature = "dotenv")]
	#[from]
	EnvParse(dotenvy::Error),

	#[cfg(not(feature = "dotenv"))]
	EnvParse(Infallible),

	CaptionsUnavailable(YTUrl),

	#[from]
	YtUrl(yt_url::Error),

	#[from]
	Reqwest(reqwest::Error),
	#[from]
	Timeout(Elapsed),
	#[from]
	Join(JoinError),
	#[from]
	Io(std::io::Error),
	#[from]
	Sqlx(sqlx::Error),

	#[from]
	Custom(String),
}

impl Error {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}
}

impl From<&str> for Error {
	fn from(value: &str) -> Self {
		Self::Custom(value.to_string())
	}
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ErrorMessage {
	pub error: bool,
	pub message: String,
}
fn error_response(status: StatusCode, s: impl Into<String>) -> (StatusCode, Json<ErrorMessage>) {
	(
		status,
		Json(ErrorMessage {
			error: true,
			message: s.into(),
		}),
	)
}

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		use Error as E;
		event!(Level::WARN, error=?self);

		match self {
			E::EnvMissingOrInvalid(_) | E::EnvParse(_) => {
				error_response(StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
			}
			E::Timeout(_) => error_response(StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout"),
			E::YtUrl(e) => error_response(StatusCode::BAD_REQUEST, e.to_string()),
			E::CaptionsUnavailable(url) => error_response(
				StatusCode::NOT_FOUND,
				format!("Captions not available for URL: {}", url.as_str()),
			),
			E::Sqlx(_) | E::Reqwest(_) | E::Join(_) | E::Io(_) | E::Custom(_) => {
				error_response(StatusCode::INTERNAL_SERVER_ERROR, "Unhandled Server Error")
			}
		}
		.into_response()
	}
}

impl Display for Error {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		write!(formatter, "{self:?}")
	}
}

impl std::error::Error for Error {}
