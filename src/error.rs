use crate::web::utils::YTUrl;
use axum::response::IntoResponse;
use derive_more::From;
use reqwest::StatusCode;
use tokio::{task::JoinError, time::error::Elapsed};
use tracing::{event, Level};
use url::Url;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	EnvMissing(&'static str),
	#[from]
	EnvParse(dotenvy::Error),

	CaptionsUnavailable(YTUrl),
	UnsupportedUrl(Url),

	#[from]
	Reqwest(reqwest::Error),
	#[from]
	Url(url::ParseError),
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

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		use Error as E;
		event!(Level::WARN, error=?self);
		match self {
			E::EnvMissing(_) | E::EnvParse(_) => {
				(StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response()
			}
			E::Timeout(_) => (StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout").into_response(),
			E::Url(_) => (StatusCode::BAD_REQUEST, "Invalid URL").into_response(),
			E::UnsupportedUrl(url) => {
				(StatusCode::BAD_REQUEST, format!("Unsupported URL: {url}")).into_response()
			}
			E::CaptionsUnavailable(url) => (
				StatusCode::NOT_FOUND,
				format!("Captions not available for URL: {}", url.as_str()),
			)
				.into_response(),
			E::Sqlx(_) | E::Reqwest(_) | E::Join(_) | E::Io(_) | E::Custom(_) => {
				(StatusCode::INTERNAL_SERVER_ERROR, "Unhandled Server Error").into_response()
			}
		}
	}
}
