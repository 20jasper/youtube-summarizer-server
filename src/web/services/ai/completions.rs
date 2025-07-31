use derive_builder::Builder;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::web::services::ai::deepinfra::{Message, Response};

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
	token: String,
}

impl CompletionClient {
	pub fn new(token: impl Into<String>, url: Url, model: impl Into<String>) -> Self {
		Self {
			model: model.into(),
			url,
			token: token.into(),
		}
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
			.bearer_auth(&self.token)
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
