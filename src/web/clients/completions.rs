use crate::error::{Error, Result};
use crate::web::services::env::load_env;
use derive_builder::Builder;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
	pub role: String,
	pub content: String,
}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Choice {
	pub message: Message,
}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Response {
	pub choices: Vec<Choice>,
}

/// more documentation can be found here <https://deepinfra.com/meta-llama/Meta-Llama-3.1-70B-Instruct/api?version=25acb1b514688b222a02a89c6976a8d7ad0e017f#input-model>
#[derive(Clone, Serialize, Deserialize, Default, Debug, Builder, PartialEq)]
#[builder(pattern = "mutable")]
#[builder(derive(Debug))]
#[builder(setter(into, strip_option), default)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct CompletionRequest {
	model: String,
	messages: Vec<Message>,
	#[serde(skip_serializing_if = "Option::is_none")]
	max_tokens: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	stream: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	temperature: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	top_p: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	top_k: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	n: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	presence_penalty: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	frequency_penalty: Option<f32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	repetition_penalty: Option<f32>,
}

impl CompletionRequestBuilder {
	fn validate(&self) -> std::result::Result<(), String> {
		if self
			.model
			.as_ref()
			.is_none_or(String::is_empty)
		{
			return Err("Model cannot be empty".into());
		}
		if self
			.messages
			.as_ref()
			.is_none_or(Vec::is_empty)
		{
			return Err("Messages cannot be empty".into());
		}
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionClient {
	model: String,
	url: Url,
	api_key: String,
}

impl CompletionClient {
	pub fn from_env() -> Result<Self> {
		const OPEN_AI_API_KEY: &str = "OPEN_AI_API_KEY";
		const OPEN_AI_MODEL: &str = "OPEN_AI_MODEL";
		const OPEN_AI_BASE_URL: &str = "OPEN_AI_BASE_URL";
		const COMPLETIONS_PATH: &str = "chat/completions";

		load_env()?;

		let api_key = env::var(OPEN_AI_API_KEY).map_err(|_| Error::EnvMissing(OPEN_AI_API_KEY))?;
		let model = env::var(OPEN_AI_MODEL).map_err(|_| Error::EnvMissing(OPEN_AI_MODEL))?;
		let url = env::var(OPEN_AI_BASE_URL)
			.map_err(|_| Error::EnvMissing(OPEN_AI_BASE_URL))?
			.parse::<Url>()
			.and_then(|x| x.join(COMPLETIONS_PATH))?;

		Ok(Self {
			model,
			url,
			api_key,
		})
	}

	pub async fn post(&self, prompt: &str, text: &str) -> Result<String> {
		let payload = CompletionRequestBuilder::default()
			.model(&self.model)
			.max_tokens(1000_u32)
			.messages([
				Message {
					role: "system".into(),
					content: prompt.into(),
				},
				Message {
					role: "user".into(),
					content: text.into(),
				},
			])
			.build()
			.map_err(|e| format!("couldn't build completion request: {e:?}"))?;

		let response = Client::new()
			.post(self.url.as_ref())
			.bearer_auth(&self.api_key)
			.json(&payload)
			.send()
			.await?;

		let json = response.json::<Response>().await?;
		let content = json
			.choices
			.first()
			.unwrap()
			.message
			.content
			.clone();
		Ok(content)
	}
}
