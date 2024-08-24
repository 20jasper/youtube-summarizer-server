use core::fmt::{self, Display, Formatter};

use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	LoginFail,

	// Model errors
	TicketDeleteFailureIdNotFound { id: u64 },
}

impl IntoResponse for Error {
	/// blanket implementation so server errors are not exposed to the client
	fn into_response(self) -> axum::response::Response {
		(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED SERVER ERROR").into_response()
	}
}

impl Display for Error {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		write!(formatter, "{self:?}")
	}
}

impl std::error::Error for Error {}
