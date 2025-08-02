//! low level wrapper over `yt-dlp` CLI
//!
//! Abstracts implementation details like file system reads

mod cache;

use crate::error::{Error, Result};
use crate::web::utils::YTUrl;
use derive_builder::Builder;
use reqwest::Url;
use std::process::Command;
use std::process::Output;

const FLAG_NO_SIMULATE: &str = "--no-simulate";
const FLAG_SKIP_DOWNLOAD: &str = "--skip-download";
const FLAG_RETRIES: &str = "--retries";
const FLAG_OUTPUT: &str = "--output";
const FLAG_PROXY: &str = "--proxy";
const FLAG_PATHS: &str = "--paths";
const FLAG_WRITE_SUBS: &str = "--write-subs";
const FLAG_WRITE_AUTO_SUBS: &str = "--write-auto-subs";
const FLAG_SUB_LANGS: &str = "--sub-langs";
const FLAG_SUB_FORMAT: &str = "--sub-format";

#[derive(Clone, Debug, Builder, PartialEq)]
pub struct YtdlpClient {
	#[builder(default)]
	download_subtitles: bool,
	#[builder(default = 3)]
	retries: u8,
	proxy: Url,
}

impl YtdlpClient {
	fn base_cmd(&self) -> Command {
		const YTDLP: &str = "yt-dlp";
		/// <https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#output-template-examples>
		const OUTPUT_TEMPLATE: &str = "%(id)s";

		let mut cmd = Command::new(YTDLP);
		cmd.arg(FLAG_NO_SIMULATE)
			.arg(FLAG_SKIP_DOWNLOAD)
			.arg(FLAG_RETRIES)
			.arg(self.retries.to_string())
			.arg(FLAG_OUTPUT)
			.arg(OUTPUT_TEMPLATE)
			.arg(FLAG_PROXY)
			.arg(self.proxy.as_str())
			.arg(FLAG_PATHS)
			.arg(
				cache::get_artifact_dir()
					.as_path()
					.to_str()
					.expect("path should always be valid utf8"),
			);

		if self.download_subtitles {
			cmd.arg(FLAG_WRITE_SUBS)
				.arg(FLAG_WRITE_AUTO_SUBS)
				.arg(FLAG_SUB_LANGS)
				.arg("en*")
				.arg(FLAG_SUB_FORMAT)
				.arg("vtt");
		}

		cmd
	}

	pub fn request(&self, url: &YTUrl) -> Result<String> {
		let mut cmd = self.base_cmd();

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
		let transcript = cache::get(url).ok_or(Error::CaptionsUnavailable(url.clone()));
		// to not waste file system space
		cache::delete(url)?;

		transcript
	}
}
