use crate::web::utils::YTUrl;
use crate::{
	error::{Error, Result},
	web::services::{
		cache::{self, Key},
		env::load_env,
		transcript::TranscriptState,
	},
};
use derive_builder::Builder;
use reqwest::Url;
use std::process::Command;
use std::{env, process::Output};

#[derive(Clone, Debug, Builder, PartialEq)]
struct YTDLPCommandSettings {
	#[builder(default)]
	download_subtitles: bool,
	#[builder(default = 3)]
	retries: u8,
	proxy: Url,
}

struct YTDLPCommand;

impl YTDLPCommand {
	fn base(settings: &YTDLPCommandSettings) -> Command {
		const YTDLP: &str = "yt-dlp";
		/// <https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#output-template-examples>
		const OUTPUT_TEMPLATE: &str = "%(id)s";

		let mut cmd = Command::new(YTDLP);
		cmd.arg("--no-simulate")
			.arg("--skip-download")
			.arg("--retries")
			.arg(settings.retries.to_string())
			.arg("--output")
			.arg(OUTPUT_TEMPLATE)
			.arg("--proxy")
			.arg(settings.proxy.as_str())
			.arg("--paths")
			.arg(
				cache::get_artifact_dir()
					.as_path()
					.to_str()
					.expect("path should always be valid utf8"),
			);

		if settings.download_subtitles {
			cmd.arg("--write-subs")
				.arg("--write-auto-subs")
				.arg("--sub-langs")
				.arg("en*")
				.arg("--sub-format")
				.arg("vtt");
		}

		cmd
	}

	fn request(settings: &YTDLPCommandSettings, url: &YTUrl) -> Result<String> {
		let mut cmd = Self::base(settings);

		cmd.arg(url.as_str());

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

		// ytdlp will write to a file in the output dir
		cache::get(&Key {
			url: url.clone(),
			state: TranscriptState::Raw,
		})
		.ok_or(Error::CaptionsUnavailable(url.clone()))
	}
}

pub struct YTClient {
	retries: u8,
	proxy: Url,
}

impl YTClient {
	pub fn from_env() -> Result<Self> {
		const YOUTUBE_PROXY: &str = "YOUTUBE_PROXY";
		const YOUTUBE_RETRIES: &str = "YOUTUBE_RETRIES";
		load_env()?;

		let proxy = env::var(YOUTUBE_PROXY).map_err(|_| Error::EnvMissing(YOUTUBE_PROXY))?;
		let proxy = Url::parse(&proxy)?;
		let retries = env::var(YOUTUBE_RETRIES)
			.ok()
			.and_then(|s| s.parse::<u8>().ok())
			.unwrap_or(3);

		Ok(Self { retries, proxy })
	}

	pub fn fetch_captions(&self, url: &YTUrl) -> Result<String> {
		if let Some(transcript) = cache::get(&Key {
			url: url.clone(),
			state: TranscriptState::Raw,
		}) {
			return Ok(transcript);
		}

		YTDLPCommand::request(
			&YTDLPCommandSettingsBuilder::default()
				.proxy(self.proxy.clone())
				.retries(self.retries)
				.download_subtitles(true)
				.build()
				.unwrap(),
			url,
		)
	}
}
