use core::net::{Ipv4Addr, SocketAddr};
use reqwest::StatusCode;
use sqlx::{PgConnection, PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;
use youtube_summarizer_server::web::services::env::{
	ApplicationSettings, DatabaseSettings, FromEnv as _,
};

#[tokio::test]
#[rstest::rstest]
#[case()]
#[case()]
#[case()]
#[case()]
#[case()]
async fn spawns_non_conflicting_app_instances() {
	let TestApp { addr, .. } = spawn_app().await;

	let res = reqwest::Client::default()
		.get(format!("{addr}/"))
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);
}

struct TestApp {
	addr: String,
	#[allow(dead_code, reason = "will use it later!")]
	pool: PgPool,
}

pub async fn setup_test_db(settings: &DatabaseSettings) -> sqlx::Pool<sqlx::Postgres> {
	use sqlx::Connection;
	use sqlx::Executor;

	{
		let maintenance_settings = DatabaseSettings {
			name: "postgres".into(),
			..settings.clone()
		};

		PgConnection::connect(&maintenance_settings.connection_string())
			.await
			.expect("failed to connect to postgres")
			.execute(format!(r#"CREATE DATABASE "{}""#, settings.name).as_str())
			.await
			.expect("failed to make new DB")
	};

	let pool = PgPool::connect(&settings.connection_string())
		.await
		.expect("failed to connect to new db");

	sqlx::migrate!("./migrations")
		.run(&pool)
		.await
		.expect("failed to migrate db");

	pool
}

async fn spawn_app() -> TestApp {
	let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
		.await
		.unwrap();
	let addr = listener.local_addr().unwrap();

	let mut db_settings = DatabaseSettings::from_env().unwrap();
	db_settings.name = Uuid::new_v4().to_string();

	let pool = setup_test_db(&db_settings).await;

	let server = youtube_summarizer_server::run(
		listener,
		pool.clone(),
		ApplicationSettings {
			// Both don't matter
			port: 0,
			public_dir: "lskdjfdsl".into(),
		},
	);

	tokio::spawn(server);

	TestApp {
		addr: format!("http://{addr}",),
		pool,
	}
}
