use crate::web::clients::yt_dlp::metadata::Chapter;

const METADATA_TEMPLATE: &str = include_str!("./prompts/metadata.xml");

#[derive(Debug, Clone, PartialEq)]
pub struct PromptContext {
	title: String,
	chapters: Vec<Chapter>,
}

impl PromptContext {
	pub fn new(title: impl Into<String>, chapters: impl Into<Vec<Chapter>>) -> Self {
		Self {
			title: title.into(),
			chapters: chapters.into(),
		}
	}

	pub const ONESHOT_SUMMARY: &str = include_str!("./prompts/summary/oneshot.md");
	pub const CHUNKED_SUMMARY: &str = include_str!("./prompts/summary/chunks/chunk.md");
	pub const CHUNKED_COMBINE: &str = include_str!("./prompts/summary/chunks/combine.md");

	fn render(&self, template: &str) -> String {
		template
			.replace("{title}", &self.title)
			.replace(
				"{chapters}",
				&self
					.chapters
					.iter()
					.map(ToString::to_string)
					.collect::<Vec<_>>()
					.join("\n"),
			)
	}

	pub fn metadata(&self) -> String {
		self.render(METADATA_TEMPLATE)
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::indexing_slicing)]
	use super::*;

	#[test]
	fn oneshot_prompt_includes_title_tag() {
		let title = "Waffle House";
		let chapters = vec![
			Chapter {
				title: Some("Intro".into()),
				start_time: 0.0,
				end_time: Some(30.0),
			},
			Chapter {
				title: Some("Outro".into()),
				start_time: 270.0,
				end_time: None,
			},
		];

		let rendered = PromptContext::new(title, chapters.clone()).metadata();
		assert!(rendered.contains(title));
		assert!(!rendered.contains("{title}"));

		assert!(rendered.contains(&chapters[0].to_string()));
		assert!(rendered.contains(&chapters[1].to_string()));
		assert!(!rendered.contains("{chapters}"));
	}
}
