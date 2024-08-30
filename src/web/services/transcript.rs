use core::str;
use regex::Regex;
use reqwest::Url;
use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use std::error::Error;
pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

const YTDLP: &str = "yt-dlp";
const RETRIES: &str = "10";
/// <https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#output-template-examples>
const OUTPUT_TEMPLATE: &str = "%(id)s";

pub fn get_by_url(url: &str) -> Result<String> {
	let output_path = env::var("OUTPUT_PATH").unwrap_or_else(|_| "./transcripts".to_string());

	let mut cmd = Command::new(YTDLP);
	let cmd = cmd.args([
		"--print",
		"filename",
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
		"--paths",
		&output_path,
		"-i",
		url,
		"--username",
		"oauth2",
		"--password",
		"''",
	]);

	let Output {
		status,
		stdout,
		stderr,
	} = cmd.output()?;

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

	let stdout = str::from_utf8(&stdout)?;
	let mut path = PathBuf::from(stdout.trim_end());
	path.set_extension("en.vtt");

	let transcript = fs::read_to_string(&path)
		.map_err(|e| format!("could not find path {}: {e}", path.display()))?;
	Ok(transcript)
}

pub fn authorize() -> Result<()> {
	let mut cmd = Command::new(YTDLP);
	cmd.args([
		"--username",
		"oauth2",
		"--password",
		"''",
		"https://www.youtube.com/",
	]);

	cmd.stdout(Stdio::inherit()).output()?;

	Ok(())
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

pub fn get_write_path(url: &str) -> Result<PathBuf> {
	let write_dir = env::var("WRITE_DIR").unwrap_or_else(|_| "./dist".to_string());

	let parsed_url = url.parse::<Url>()?;
	let video_id = parsed_url
		.query_pairs()
		.find(|(key, _)| key == "v")
		.map_or("default".into(), |(_, id)| id);
	let write_dir = PathBuf::from(write_dir);
	let mut write_path = write_dir.join(video_id.into_owned());
	write_path.set_extension("md");

	Ok(write_path)
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
