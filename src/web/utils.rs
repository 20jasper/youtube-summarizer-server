use crate::error::{Error, Result};
use reqwest::Url;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YTUrl {
	url: Url,
}

impl YTUrl {
	pub fn parse_from_url(url: Url) -> Result<Self> {
		const YOUTUBE: &str = "youtube";
		if let Some(host) = url.host_str()
			&& host.contains(YOUTUBE)
		{
			Ok(Self { url })
		} else {
			Err(Error::UnsupportedUrl(url))
		}
	}

	pub fn parse_from_str(url: &str) -> Result<Self> {
		let url = Url::parse(url)?;
		Self::parse_from_url(url)
	}

	pub fn as_url(&self) -> &Url {
		&self.url
	}

	pub fn as_str(&self) -> &str {
		self.url.as_str()
	}

	pub fn id(&self) -> Option<Cow<'_, str>> {
		self.as_url()
			.query_pairs()
			.find(|(key, _)| key == "v")
			.map(|(_, id)| id)
	}

	pub fn id_string(&self) -> Option<String> {
		self.id()
			.map(std::borrow::Cow::into_owned)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const YT_URL: &str =
		"https://www.youtube.com/watch?v=DjcC6p_8fpE&pp=ygUWamFjb2IgYXNwZXIgdHlwZXNjcmlwdA%3D%3D";
	const YT_ID: &str = "DjcC6p_8fpE";

	#[test]
	fn should_get_video_id() -> Result<()> {
		let url = YTUrl::parse_from_str(YT_URL)?;

		assert_eq!(url.id(), Some(YT_ID.into()));

		Ok(())
	}

	#[test]
	fn invalid_url() {
		let invalid_url = "https://lasagna.com/watch?v=DjcC6p_8fpE";
		YTUrl::parse_from_str(invalid_url).unwrap_err();
	}
}
