use http_body_util::BodyExt as _;
use mockall::mock;
use youtube_summarizer_server::web::clients::CompletionClient;

mock! {
	pub Completion {}
	#[allow(refining_impl_trait, reason="test-only mock return impls")]
	impl CompletionClient for Completion {
		fn post<'a>(&self, prompt: &'a str, user_messages: &'a [std::string::String]) -> impl core::future::Future<Output = youtube_summarizer_server::error::Result<String>> + Send;
		fn post_stream<'a>(self, prompt: &'a str, user_messages: &'a [std::string::String]) -> impl core::future::Future<Output = youtube_summarizer_server::error::Result<core::pin::Pin<Box<dyn futures::Stream<Item = youtube_summarizer_server::web::clients::completions::stream::SseMessage> + Send>>>> + Send;
	}

	impl Clone for Completion {
		fn clone(&self) -> Self;
	}
}

#[allow(
	dead_code,
	reason = "It is used, but each integration test is compiled separately, so it considered unused if not imported in each test file"
)]
pub async fn body_to_string(res: axum::response::Response) -> Result<String> {
	let bytes = res
		.into_body()
		.collect()
		.await?
		.to_bytes()
		.to_vec();
	Ok(String::from_utf8(bytes)?)
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
