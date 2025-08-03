pub use yt_url::YTUrl;

pub mod yt_url {
	pub use crate::web::utils::yt_url::error::{Error, Result};
	use reqwest::Url;
	use std::borrow::Cow;

	const VIDEO_PARAM: &str = "v";

	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct YTUrl {
		url: Url,
		id: String,
	}

	impl TryFrom<Url> for YTUrl {
		type Error = Error;

		fn try_from(url: Url) -> Result<Self> {
			const YOUTUBE: &str = "youtube";
			if let Some(host) = url.host_str()
				&& host.contains(YOUTUBE)
				&& let Some(id) = Self::try_get_id(&url)
			{
				let id = id.to_string();
				Ok(Self { url, id })
			} else {
				Err(Error::Unsupported(url.into()))
			}
		}
	}

	impl TryFrom<&str> for YTUrl {
		type Error = Error;

		fn try_from(url: &str) -> Result<Self> {
			let url = Url::parse(url).map_err(|_| Error::Invalid(url.into()))?;
			Self::try_from(url)
		}
	}

	impl YTUrl {
		pub fn as_url(&self) -> &Url {
			&self.url
		}

		pub fn as_str(&self) -> &str {
			self.url.as_str()
		}

		fn try_get_id(url: &Url) -> Option<Cow<'_, str>> {
			url.query_pairs()
				.find(|(key, _)| key == VIDEO_PARAM)
				.map(|(_, id)| id)
		}

		pub fn id(&self) -> &str {
			&self.id
		}
	}

	pub mod error {

		#[derive(Debug, Clone, PartialEq, Eq)]
		pub enum Error {
			Unsupported(String),
			Invalid(String),
		}

		pub type Result<T> = std::result::Result<T, Error>;

		impl std::fmt::Display for Error {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					Error::Unsupported(url) => write!(f, "Unsupported URL: {url}"),
					Error::Invalid(url) => write!(f, "Invalid URL: {url}"),
				}
			}
		}

		impl std::error::Error for Error {}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		const YT_URL: &str = "https://www.youtube.com/watch?v=DjcC6p_8fpE&pp=ygUWamFjb2IgYXNwZXIgdHlwZXNjcmlwdA%3D%3D";
		const YT_ID: &str = "DjcC6p_8fpE";

		#[test]
		fn should_get_video_id() -> Result<()> {
			let url = YTUrl::try_from(YT_URL)?;

			assert_eq!(url.id(), YT_ID);

			Ok(())
		}

		#[test]
		fn v_param_later() -> Result<()> {
			let url = "https://www.youtube.com/watch?t=998s&v=G-aXK-iOpnA";

			YTUrl::try_from(url)?;

			Ok(())
		}

		#[test]
		fn invalid_host() {
			let invalid_url = "https://lasagna.com/watch?v=DjcC6p_8fpE";
			YTUrl::try_from(invalid_url).unwrap_err();
		}

		#[test]
		fn missing_v_param() {
			let invalid_url = "https://youtube.com/watch";
			YTUrl::try_from(invalid_url).unwrap_err();
		}
	}
}
