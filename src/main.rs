use core::net::SocketAddr;

use axum::{middleware, response::Response, serve, Router};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub mod error;
pub mod model;
pub mod web;

pub use self::error::{Error, Result};
use web::routes::{greeting, login};

async fn response_mapper(res: Response) -> Response {
	println!("Hello from the Response Mapper");
	println!();

	res
}

#[tokio::main]
async fn main() {
	let routes = Router::new()
		.merge(greeting::routes())
		.merge(login::routes())
		// layers run from bottom to top
		.layer(middleware::map_response(response_mapper))
		.layer(tower_cookies::CookieManagerLayer::new())
		.fallback_service(ServeDir::new("public/"));

	let address = SocketAddr::from(([127, 0, 0, 1], 8080));
	let listener = TcpListener::bind(address)
		.await
		.unwrap();
	println!("Listening on http://{address}");

	serve(listener, routes.into_make_service())
		.await
		.unwrap();
}
