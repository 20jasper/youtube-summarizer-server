pub use error::{Error, Result};
use secrecy::{ExposeSecret, SecretString};
use std::path::PathBuf;

mod error {
	#[cfg(not(feature = "dotenv"))]
	use core::convert::Infallible;

	use core::fmt::{self, Display, Formatter};
	use derive_more::From;

	pub type Result<T> = core::result::Result<T, Error>;

	#[derive(Debug, From)]
	pub enum Error {
		#[cfg(feature = "dotenv")]
		#[from]
		Load(dotenvy::Error),
		#[cfg(not(feature = "dotenv"))]
		Load(Infallible),

		#[from]
		Parse(envy::Error),
	}

	impl Display for Error {
		fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
			write!(formatter, "{self:?}")
		}
	}

	impl std::error::Error for Error {}
}

#[cfg(not(feature = "dotenv"))]
pub fn load_env() -> Result<()> {
	Ok(())
}

#[cfg(feature = "dotenv")]
use dotenvy::dotenv;
use url::Url;
#[cfg(feature = "dotenv")]
pub fn load_env() -> Result<()> {
	dotenv()?;
	Ok(())
}

pub trait FromEnv {
	fn from_env() -> Result<Self>
	where
		Self: Sized;
}

macro_rules! from_env {
	($t:ty, $prefix:literal) => {
		impl FromEnv for $t {
			fn from_env() -> Result<Self> {
				load_env()?;
				Ok(envy::prefixed($prefix).from_env()?)
			}
		}
	};
}

from_env!(YouTubeSettings, "YOUTUBE_");
from_env!(DatabaseSettings, "DB_");
from_env!(OpenAISettings, "OPEN_AI_");
from_env!(AxiomSettings, "AXIOM_");
from_env!(ApplicationSettings, "APPLICATION_");
from_env!(RustSettings, "RUST_");

fn output_path() -> PathBuf {
	"./transcripts".into()
}
fn retries() -> u8 {
	3
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct YouTubeSettings {
	pub proxy: Url,
	#[serde(default = "retries")]
	pub retries: u8,
	#[serde(default = "output_path")]
	pub output_path: PathBuf,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
	pub username: String,
	pub password: SecretString,
	pub port: u16,
	pub host: String,
	pub name: String,
}

impl DatabaseSettings {
	pub fn connection_string(&self) -> SecretString {
		format!(
			"postgres://{}:{}@{}:{}/{}",
			self.username,
			self.password.expose_secret(),
			self.host,
			self.port,
			self.name
		)
		.into()
	}
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OpenAISettings {
	pub api_key: SecretString,
	pub base_url: Url,
	pub model: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AxiomSettings {
	pub token: SecretString,
	pub dataset: String,
}

fn rust_log() -> String {
	"youtube_summarizer_server=trace,tower_http=debug,reqwest=debug".into()
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct RustSettings {
	#[serde(default = "rust_log")]
	pub log: String,
}

fn port() -> u16 {
	8080
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
	#[serde(default = "port")]
	pub port: u16,
	pub public_dir: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Settings {
	pub application: ApplicationSettings,
	pub database: DatabaseSettings,
	pub open_ai: OpenAISettings,
	pub youtube: YouTubeSettings,
	pub axiom: Option<AxiomSettings>,
	pub rust: RustSettings,
}

impl FromEnv for Settings {
	fn from_env() -> Result<Self> {
		Ok(Settings {
			application: ApplicationSettings::from_env()?,
			database: DatabaseSettings::from_env()?,
			open_ai: OpenAISettings::from_env()?,
			youtube: YouTubeSettings::from_env()?,
			axiom: AxiomSettings::from_env().ok(),
			rust: RustSettings::from_env()?,
		})
	}
}
