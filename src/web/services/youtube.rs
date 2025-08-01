use crate::{
	error::{Error, Result},
	web::services::{
		cache::{self, Key},
		env::load_env,
		transcript::TranscriptState,
	},
};
use reqwest::Url;
use std::process::Command;
use std::{env, process::Output};

pub fn get_video_id(url: &str) -> Option<String> {
	url.parse::<Url>()
		.ok()?
		.query_pairs()
		.find(|(key, _)| key == "v")
		.map(|(_, id)| id.into_owned())
}

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

	pub fn fetch_captions(&self, url: &Url) -> Result<String> {
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

		read_cache().ok_or("Transcript not found in cache".into())
	}
}
