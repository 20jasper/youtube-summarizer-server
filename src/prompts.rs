pub const ONESHOT_SUMMARY_TEMPLATE: &str = include_str!("./prompts/summary/oneshot.md");

pub const CHUNKED_SUMMARY_TEMPLATE: &str = include_str!("./prompts/summary/chunks/chunk.md");
pub const CHUNKED_COMBINE_TEMPLATE: &str = include_str!("./prompts/summary/chunks/combine.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptContext {
	title: String,
}

impl PromptContext {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
		}
	}

	fn render(&self, template: &str) -> String {
		template.replace("{title}", &self.title)
	}

	pub fn oneshot_prompt(&self) -> String {
		self.render(ONESHOT_SUMMARY_TEMPLATE)
	}

	pub fn chunk_prompt(&self) -> String {
		self.render(CHUNKED_SUMMARY_TEMPLATE)
	}

	pub fn combine_prompt(&self) -> String {
		self.render(CHUNKED_COMBINE_TEMPLATE)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn has_video_title_tag(s: &str, title: &str) -> bool {
		s.contains(&format!("<videoTitle>{title}</videoTitle>"))
	}

	#[test]
	fn oneshot_prompt_includes_title_tag() {
		let title = "Waffle House";
		let rendered = PromptContext::new(title).oneshot_prompt();
		assert!(has_video_title_tag(&rendered, title));
		assert!(!rendered.contains("{title}"));
	}

	#[test]
	fn chunk_prompt_includes_title_tag() {
		let title = "Lechunk";
		let rendered = PromptContext::new(title).chunk_prompt();
		assert!(has_video_title_tag(&rendered, title));
		assert!(!rendered.contains("{title}"));
	}

	#[test]
	fn combine_prompt_includes_title_tag() {
		let title = "Bungalow";
		let rendered = PromptContext::new(title).combine_prompt();
		assert!(has_video_title_tag(&rendered, title));
		assert!(!rendered.contains("{title}"));
	}
}
