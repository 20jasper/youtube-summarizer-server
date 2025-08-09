use crate::error::{Error, Result};
use crate::web::clients::completions::stream::bytes_to_event;
use crate::web::services::env::load_env;
use axum::body::Bytes;
use axum::response::sse;
use derive_builder::Builder;
use futures::{Stream, StreamExt};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::env;

pub mod stream;

mod oneshot {
	use crate::web::clients::completions::Message;
	use serde::Deserialize;

	#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
	struct Choice {
		message: Message,
	}
	#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
	pub struct Response {
		choices: (Choice,),
	}

	impl Response {
		pub fn content(self) -> String {
			self.choices.0.message.content
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
	pub role: String,
	pub content: String,
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

pub trait CompletionClient {
	fn post(&self, prompt: &str, text: &str) -> impl Future<Output = Result<String>> + Send;
	fn post_stream(
		self,
		prompt: &str,
		text: &str,
	) -> impl Future<Output = impl Stream<Item = sse::Event> + Send>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepInfraClient {
	model: String,
	url: Url,
	api_key: String,
}

impl DeepInfraClient {
	pub fn build(model: String, base_url: &Url, api_key: String) -> Result<Self> {
		const COMPLETIONS_PATH: &str = "chat/completions";
		Ok(Self {
			model,
			url: base_url
				.join(COMPLETIONS_PATH)
				.map_err(|_| {
					format!(
						"could not parse completions endpoint: {}",
						base_url.as_str()
					)
				})?,
			api_key,
		})
	}

	pub fn from_env() -> Result<Self> {
		const OPEN_AI_API_KEY: &str = "OPEN_AI_API_KEY";
		const OPEN_AI_MODEL: &str = "OPEN_AI_MODEL";
		const OPEN_AI_BASE_URL: &str = "OPEN_AI_BASE_URL";

		load_env()?;

		let api_key =
			env::var(OPEN_AI_API_KEY).map_err(|_| Error::EnvMissingOrInvalid(OPEN_AI_API_KEY))?;
		let model =
			env::var(OPEN_AI_MODEL).map_err(|_| Error::EnvMissingOrInvalid(OPEN_AI_MODEL))?;
		let url = env::var(OPEN_AI_BASE_URL)
			.map_err(|_| Error::EnvMissingOrInvalid(OPEN_AI_BASE_URL))?
			.parse::<Url>()
			.map_err(|_| Error::EnvMissingOrInvalid(OPEN_AI_BASE_URL))?;

		DeepInfraClient::build(model, &url, api_key)
	}

	async fn base_post(&self, prompt: &str, text: &str, stream: bool) -> Result<reqwest::Response> {
		let payload = CompletionRequestBuilder::default()
			.model(&self.model)
			.max_tokens(700_u32)
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
			.stream(stream)
			.build()
			.map_err(|e| format!("couldn't build completion request: {e:?}"))?;

		let response = Client::new()
			.post(self.url.as_ref())
			.bearer_auth(&self.api_key)
			.json(&payload)
			.send()
			.await?;

		Ok(response)
	}
}

impl CompletionClient for DeepInfraClient {
	async fn post(&self, prompt: &str, text: &str) -> Result<String> {
		Ok(self
			.base_post(prompt, text, false)
			.await?
			.json::<oneshot::Response>()
			.await?
			.content())
	}

	async fn post_stream(self, prompt: &str, text: &str) -> impl Stream<Item = sse::Event> + Send {
		async fn filter_map_nonempty(b: reqwest::Result<Bytes>) -> Option<sse::Event> {
			let b = b.ok()?;
			if b.is_empty() {
				return None;
			}
			bytes_to_event(&b)
		}
		self.base_post(prompt, text, true)
			.await
			.unwrap()
			.bytes_stream()
			.filter_map(filter_map_nonempty)
			.chain(futures::stream::iter(
				(0..10).map(|_| stream::SseMessage::Done.into()),
			))
	}
}
