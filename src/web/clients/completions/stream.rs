use axum::response::sse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
	pub choices: (Choice,),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
	pub delta: Option<Delta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
	pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SseState {
	Message,
	Done,
	Error,
}
#[derive(Debug, Serialize)]
pub struct SseMessage {
	pub message: Option<String>,
	pub kind: SseState,
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
		return Some(
			SseMessage {
				message: None,
				kind: SseState::Done,
			}
			.into(),
		);
	}
	let json = serde_json::from_slice::<ChatCompletionChunk>(bytes);

	if let Ok(ChatCompletionChunk {
		choices: (Choice {
			delta: Some(Delta { content }),
		},),
	}) = json
	{
		Some(
			SseMessage {
				message: Some(content),
				kind: SseState::Message,
			}
			.into(),
		)
	} else {
		None
	}
}
