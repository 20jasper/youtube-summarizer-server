use crate::error::{Error, Result};
use reqwest::Url;
use std::borrow::Cow;

const VIDEO_PARAM: &str = "v";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YTUrl {
	url: Url,
	id: String,
}

impl YTUrl {
	pub fn parse_from_url(url: Url) -> Result<Self> {
		const YOUTUBE: &str = "youtube";
		if let Some(host) = url.host_str()
			&& host.contains(YOUTUBE)
			&& let Some(id) = Self::try_get_id(&url)
		{
			let id = id.to_string();
			Ok(Self { url, id })
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

	fn try_get_id(url: &Url) -> Option<Cow<'_, str>> {
		url.query_pairs()
			.find(|(key, _)| key == VIDEO_PARAM)
			.map(|(_, id)| id)
	}

	pub fn id(&self) -> &str {
		&self.id
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

		assert_eq!(url.id(), YT_ID);

		Ok(())
	}

	#[test]
	fn invalid_host() {
		let invalid_url = "https://lasagna.com/watch?v=DjcC6p_8fpE";
		YTUrl::parse_from_str(invalid_url).unwrap_err();
	}

	#[test]
	fn missing_v_param() {
		let invalid_url = "https://youtube.com/watch";
		YTUrl::parse_from_str(invalid_url).unwrap_err();
	}
}
