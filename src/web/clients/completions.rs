use crate::error::Result;
use crate::web::clients::completions::stream::{SseMessage, bytes_to_sse_message};
use crate::web::services::env::{FromEnv, OpenAISettings};
use core::future::Future;
use core::iter;
use core::pin::Pin;
use derive_builder::Builder;
use futures::{Stream, StreamExt};
use mockall::automock;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

const SYSTEM_ROLE: &str = "system";
const USER_ROLE: &str = "user";

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

#[automock]
pub trait CompletionClient: Send + Sync {
	fn post<'a>(
		&self,
		prompt: &'a str,
		user_messages: &'a [String],
	) -> impl Future<Output = Result<String>> + Send;
	fn post_stream<'a>(
		self,
		prompt: &'a str,
		user_messages: &'a [String],
	) -> impl Future<Output = Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>>>;
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
		let OpenAISettings {
			api_key,
			base_url,
			model,
		} = OpenAISettings::from_env()?;

		DeepInfraClient::build(model, &base_url, api_key)
	}

	async fn base_post(
		&self,
		prompt: &str,
		user_messages: &[String],
		stream: bool,
	) -> Result<reqwest::Response> {
		let system = iter::once(Message {
			role: SYSTEM_ROLE.into(),
			content: prompt.into(),
		});

		let user = user_messages
			.iter()
			.map(|text| Message {
				role: USER_ROLE.into(),
				content: text.clone(),
			});
		let messages: Vec<_> = system.chain(user).collect();

		let payload = CompletionRequestBuilder::default()
			.model(&self.model)
			.max_tokens(700_u32)
			.messages(messages)
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
	async fn post(&self, prompt: &str, user_messages: &[String]) -> Result<String> {
		Ok(self
			.base_post(prompt, user_messages, false)
			.await?
			.json::<oneshot::Response>()
			.await?
			.content())
	}

	async fn post_stream(
		self,
		prompt: &str,
		user_messages: &[String],
	) -> Result<Pin<Box<dyn Stream<Item = SseMessage> + Send>>> {
		Ok(Box::pin(
			self.base_post(prompt, user_messages, true)
				.await?
				.bytes_stream()
				.filter_map(|res| async {
					res.ok()
						.and_then(|b| bytes_to_sse_message(&b))
				})
				.chain(futures::stream::iter(0..5).map(|_| SseMessage::Done)),
		))
	}
}
