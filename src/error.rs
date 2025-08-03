use core::fmt::{self, Display, Formatter};

use crate::web::utils::{YTUrl, yt_url};
use axum::response::IntoResponse;
use derive_more::From;
use reqwest::StatusCode;
use tokio::{task::JoinError, time::error::Elapsed};
use tracing::{Level, event};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
	EnvMissingOrInvalid(&'static str),
	#[from]
	EnvParse(dotenvy::Error),

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

impl IntoResponse for Error {
	fn into_response(self) -> axum::response::Response {
		use Error as E;
		event!(Level::WARN, error=?self);
		match self {
			E::EnvMissingOrInvalid(_) | E::EnvParse(_) => {
				(StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable").into_response()
			}
			E::Timeout(_) => (StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout").into_response(),
			E::YtUrl(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
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

impl Display for Error {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		write!(formatter, "{self:?}")
	}
}

impl std::error::Error for Error {}
