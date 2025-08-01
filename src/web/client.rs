mod deepinfra;
mod youtube;

pub use youtube::YTClient;

use reqwest::Client;

pub fn base_client() -> Client {
	Client::new()
}
