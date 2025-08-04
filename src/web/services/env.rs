use crate::error::Result;

#[cfg(not(feature = "dotenv"))]
pub fn load_env() -> Result<()> {
	Ok(())
}

#[cfg(feature = "dotenv")]
use dotenvy::dotenv;
#[cfg(feature = "dotenv")]
pub fn load_env() -> Result<()> {
	dotenv()?;
	Ok(())
}
