use derive_more::From;
use displaydoc::Display;
use thiserror::Error;

#[derive(Display, Debug, Error, From)]
pub enum Error {
	/// Environment variable `{0}` is not set
	EnvMissing(String),
	/// Error parsing environment variables: {0}
	EnvParse(#[from] dotenvy::Error),

	/// Error: {0:?}
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error>),
}

pub type Result<T> = core::result::Result<T, Error>;
