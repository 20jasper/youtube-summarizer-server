use crate::{
	error::{Error, Result},
	web::services::{
		cache::{self, Key},
		env::load_env,
		transcript::TranscriptState,
	},
};
use reqwest::Url;
use std::{borrow::Cow, process::Command};
use std::{env, process::Output};

pub struct YTClient {
	pub retries: u8,
	pub proxy: Url,
}

impl YTClient {
	pub fn from_env() -> Result<Self> {
		const YOUTUBE_PROXY: &str = "YOUTUBE_PROXY";
		const YOUTUBE_RETRIES: &str = "YOUTUBE_RETRIES";
		load_env()?;

		let retries = env::var(YOUTUBE_RETRIES)
			.ok()
			.and_then(|s| s.parse::<u8>().ok())
			.unwrap_or(3);
		let proxy = env::var(YOUTUBE_PROXY).map_err(|_| Error::EnvMissing(YOUTUBE_PROXY))?;
		let proxy = Url::parse(&proxy)?;

		Ok(Self { retries, proxy })
	}

	pub fn fetch_captions(&self, url: &YTUrl) -> Result<String> {
		const YTDLP: &str = "yt-dlp";
		/// <https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#output-template-examples>
		const OUTPUT_TEMPLATE: &str = "%(id)s";

		let read_cache = || {
			cache::get(&Key {
				url: url.clone(),
				state: TranscriptState::Raw,
			})
		};
		if let Some(transcript) = read_cache() {
			return Ok(transcript);
		}

		// ytdlp will write to a file in the output dir
		let mut cmd = Command::new(YTDLP);
		let cmd = cmd.args([
			"--no-simulate",
			"--write-subs",
			"--write-auto-subs",
			"--sub-langs",
			"en*",
			"--sub-format",
			"vtt",
			"--skip-download",
			"--retries",
			self.retries.to_string().as_str(),
			"--output",
			OUTPUT_TEMPLATE,
			"--proxy",
			self.proxy.as_str(),
			"--paths",
			cache::get_artifact_dir()
				.as_path()
				.to_str()
				.expect("path should always be valid utf8"),
			"-i",
			url.as_str(),
		]);

		let Output { status, stderr, .. } = cmd.output()?;
		if !status.success() {
			return Err(format!(
				"get transcript failed with status code {}, {:?}",
				status
					.code()
					.ok_or("could not get status code")?,
				str::from_utf8(&stderr)
			)
			.into());
		}

		read_cache().ok_or(Error::CaptionsUnavailable(url.clone()))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YTUrl {
	url: Url,
}

impl YTUrl {
	pub fn parse_from_url(url: Url) -> Result<Self> {
		const YOUTUBE: &str = "youtube";
		const YOUTUBEDOTBE: &str = "youtu.be";
		if url
			.host_str()
			.is_some_and(|host| host.contains(YOUTUBE) || host.contains(YOUTUBEDOTBE))
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
