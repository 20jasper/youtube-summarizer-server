use crate::config::Config;
use crate::error::Result;
use crate::web::routes::transcript;
use crate::web::services::ai::completions::CompletionClient;
use crate::web::services::ai::prompt::ARTICLE_TEMPLATE;
use core::str;
use core::time::Duration;
use regex::Regex;
use reqwest::Url;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use tokio::time::timeout;

const YTDLP: &str = "yt-dlp";
const RETRIES: &str = "10";
/// <https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#output-template-examples>
const OUTPUT_TEMPLATE: &str = "%(id)s";
const PROXY: &str = "PROXY";

const RAW_EXT: &str = "en.vtt";
const CLEAN_EXT: &str = "en.clean";
const SUMMARY_EXT: &str = "md";

fn get_artifact_dir() -> PathBuf {
	env::var("OUTPUT_PATH")
		.unwrap_or_else(|_| "./transcripts".to_string())
		.into()
}

fn get_artifact_path(url: &str, extension: &str) -> Option<PathBuf> {
	let mut path = get_artifact_dir().join(get_video_id(url)?);
	path.set_extension(extension);

	Some(path)
}

pub async fn get_transcript_by_url(url: &str, raw: bool) -> Result<String> {
	// ytdlp will write to a file in the output dir
	let output_path = get_artifact_dir();

	// TODO try to read from cache instead
	// if fs::exists(&output_path)? {
	// 	return Ok("exists in dir already!".into());
	// }

	let proxy = env::var(PROXY).map_err(|_| "proxy is not set")?;

	let owned_url = url.to_owned();
	let join_handle = tokio::spawn(async move {
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
			RETRIES,
			"--output",
			OUTPUT_TEMPLATE,
			"--proxy",
			&proxy,
			"--paths",
			output_path
				.as_path()
				.to_str()
				.expect("path should always be valid utf8"),
			"-i",
			&owned_url,
		]);

		cmd.output()
	});

	// TODO service unavailable code if takes longer than timeout. This will depend based on proxy and server location
	let Output { status, stderr, .. } = timeout(Duration::from_secs(30), join_handle).await???;

	if !status.success() {
		return Err(format!(
			"get transcript failed with status code {}, {:?}",
			status
				.code()
				.ok_or("could not get status code")?,
			str::from_utf8(&stderr)?
		)
		.into());
	}

	let raw_path = get_artifact_path(url, RAW_EXT).expect("video id must exist at this point");

	let transcript = fs::read_to_string(&raw_path)
		.map_err(|e| format!("could not find path {}: {e}", raw_path.display()))?;

	println!("got the transcript!");
	Ok(if raw {
		transcript
	} else {
		clean_vtt(&transcript)
	})
}

pub async fn summarize_by_url(url: &str) -> Result<String> {
	get_artifact_dir();
	let transcript = get_transcript_by_url(url, false).await?;

	let clean = clean_vtt(&transcript);
	// path.set_extension("clean.en.vtt");
	// fs::write(&path, &clean).unwrap();

	let Config {
		api_key,
		model,
		base_url,
		..
	} = Config::build().unwrap();
	let client = CompletionClient::build(api_key, &base_url, model)?;
	let res = client
		.post(ARTICLE_TEMPLATE, &clean)
		.await?;

	// path.set_extension("summary.md");
	// fs::write(path, &res).unwrap();

	Ok(res)
}

/// remove timestamps and duplicate lines
pub fn clean_vtt(transcript: &str) -> String {
	let mut lines = transcript.lines();
	// skip header
	lines.find(|l| l.starts_with("Language"));

	let tags = Regex::new("</*c.*>").unwrap();
	let time_stamp = Regex::new(r"\d{2}:\d{2}:\d{2}\.\d{3}").unwrap();
	lines
		.filter(|l| !time_stamp.is_match(l))
		.map(|l| tags.replace_all(l, ""))
		.filter(|l| !l.trim().is_empty())
		.scan(Cow::from(""), |last_text, l| {
			if &l == last_text {
				Some("".into())
			} else {
				last_text.clone_from(&l);
				Some(l)
			}
		})
		.filter(|l| !l.is_empty())
		.collect::<Vec<_>>()
		.join(" ")
}

fn get_video_id(url: &str) -> Option<String> {
	url.parse::<Url>()
		.ok()?
		.query_pairs()
		.find(|(key, _)| key == "v")
		.map(|(_, id)| id.into_owned())
}

pub fn get_write_path(url: &str) -> Option<PathBuf> {
	let write_dir: PathBuf = env::var("WRITE_DIR")
		.unwrap_or_else(|_| "./dist".to_string())
		.into();
	let mut write_path = write_dir.join(get_video_id(url)?);
	write_path.set_extension("md");

	Some(write_path)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_convert_vtt_to_text() {
		let vtt = "WEBVTT
Kind: captions
Language: en
00:00:00.580 --> 00:00:01.910 align:start position:0%
[Music]
00:00:01.910 --> 00:00:01.920 align:start position:0%
[Music]

00:00:01.920 --> 00:00:04.150 align:start position:0%
[Music]
you<00:00:02.040><c> know</c><00:00:02.200><c> what's</c><00:00:02.520><c> really</c><00:00:02.840><c> not</c><00:00:03.120><c> fun</c><00:00:03.679><c> recording</c>

00:00:04.150 --> 00:00:04.160 align:start position:0%
you know what's really not fun recording

00:00:04.160 --> 00:00:06.510 align:start position:0%
you know what's really not fun recording
an<00:00:04.359><c> entire</c><00:00:04.880><c> video</c><00:00:05.160><c> for</c><00:00:05.400><c> 30</c><00:00:05.720><c> minutes</c><00:00:06.200><c> and</c><00:00:06.319><c> then</c>

00:00:06.510 --> 00:00:06.520 align:start position:0%
an entire video for 30 minutes and then
 

00:00:06.520 --> 00:00:08.669 align:start position:0%
an entire video for 30 minutes and then
realizing<00:00:07.359><c> you</c><00:00:07.520><c> forgot</c><00:00:07.839><c> to</c><00:00:08.080><c> plug</c><00:00:08.280><c> in</c><00:00:08.440><c> your</c>";

		assert_eq!(clean_vtt(vtt), "[Music] you know what's really not fun recording an entire video for 30 minutes and then");
	}

	#[test]
	fn get_path_from_url() {
		assert_eq!(
			get_write_path("https://www.youtube.com?v=gamer").unwrap(),
			PathBuf::from("./dist/gamer.md")
		);
	}
}
