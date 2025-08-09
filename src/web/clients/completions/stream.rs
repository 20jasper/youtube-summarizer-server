use axum::response::sse;
use serde::Serialize;

mod chunk {
	use serde::Deserialize;
	#[derive(Debug, Deserialize)]
	pub struct ChatCompletionChunk {
		choices: (Choice,),
	}

	impl ChatCompletionChunk {
		pub fn content(self) -> Option<String> {
			Some(self.choices.0.delta?.content)
		}
	}

	#[derive(Debug, Deserialize)]
	struct Choice {
		delta: Option<Delta>,
	}

	#[derive(Debug, Deserialize)]
	struct Delta {
		content: String,
	}
}

use chunk::ChatCompletionChunk;

#[derive(Debug, Serialize)]
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

pub fn bytes_to_event(bytes: &[u8]) -> Option<sse::Event> {
	let i = bytes.iter().position(|&x| x == b'{')?;
	let bytes = bytes.get(i..)?;

	tracing::debug!("bytes: {:?}", str::from_utf8(bytes));

	if bytes.starts_with(b"[DONE]") {
		return Some(SseMessage::Done.into());
	}
	let content = serde_json::from_slice::<ChatCompletionChunk>(bytes)
		.ok()?
		.content()?;

	Some(SseMessage::Message(content).into())
}
