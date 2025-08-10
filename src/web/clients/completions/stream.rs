use axum::response::sse;
use serde::{Deserialize, Serialize};

mod chunk {
	use serde::{Deserialize, Serialize};
	#[derive(Debug, Deserialize, Serialize)]
	pub struct ChatCompletionChunk {
		choices: (Choice,),
	}

	impl ChatCompletionChunk {
		pub fn content(self) -> Option<String> {
			Some(self.choices.0.delta?.content)
		}
	}

	#[derive(Debug, Deserialize, Serialize)]
	struct Choice {
		delta: Option<Delta>,
	}

	#[derive(Debug, Deserialize, Serialize)]
	struct Delta {
		content: String,
	}
}

use chunk::ChatCompletionChunk;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", content = "message")]
pub enum SseMessage {
	Message(String),
	Done,
	Error,
}

impl From<SseMessage> for sse::Event {
	fn from(msg: SseMessage) -> Self {
		sse::Event::default()
			.json_data(msg)
			.expect("Data was called once")
	}
}

pub fn bytes_to_sse_message(bytes: &[u8]) -> Option<SseMessage> {
	if bytes.is_empty() {
		return None;
	}

	let bytes = if let Some(rest) = bytes.strip_prefix(b"data: ") {
		rest
	} else {
		bytes
	};

	let content = serde_json::from_slice::<ChatCompletionChunk>(bytes)
		.ok()?
		.content()?;

	if content.is_empty() {
		return None;
	}

	Some(SseMessage::Message(content))
}
