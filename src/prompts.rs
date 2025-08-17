const METADATA_TEMPLATE: &str = include_str!("./prompts/metadata.xml");

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

	pub const ONESHOT_SUMMARY: &str = include_str!("./prompts/summary/oneshot.md");
	pub const CHUNKED_SUMMARY: &str = include_str!("./prompts/summary/chunks/chunk.md");
	pub const CHUNKED_COMBINE: &str = include_str!("./prompts/summary/chunks/combine.md");

	fn render(&self, template: &str) -> String {
		template.replace("{title}", &self.title)
	}

	pub fn metadata(&self) -> String {
		self.render(METADATA_TEMPLATE)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn oneshot_prompt_includes_title_tag() {
		let title = "Waffle House";
		let rendered = PromptContext::new(title).metadata();
		assert!(rendered.contains(title));
		assert!(!rendered.contains("{title}"));
	}
}
