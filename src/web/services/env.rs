use crate::error::Result;
use dotenvy::dotenv;
use std::env;

const TRUE: &str = "true";

/// Loads from .env if `LOAD_ENV` is set
pub fn load_env() -> Result<()> {
	if env::var("LOAD_ENV").is_ok_and(|x| x.eq_ignore_ascii_case(TRUE)) {
		dotenv()?;
	}
	Ok(())
}
