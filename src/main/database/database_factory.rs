use anyhow::anyhow;
use sqlx::{AnyPool, Pool, Sqlite};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

struct DatabaseFactory {
	url: bool,
}

impl DatabaseFactory {
	pub async fn create(db_type: DatabaseType) -> Result<Pool<Sqlite>, sqlx::Error> {
		let database_options = match db_type {
			DatabaseType::InMemory => Self::create_in_memory_options(),
			DatabaseType::SQLite(url) => Self::create_url_options(&url),
		};

		SqlitePoolOptions::new()
				.max_connections(5)
				.connect_with(database_options)
				.await
	}

	fn create_in_memory_options() -> SqliteConnectOptions {
		todo!()
	}

	fn create_url_options(url: &str) -> SqliteConnectOptions {
		todo!()
	}
}



enum DatabaseType {
	InMemory,
	SQLite(String),
}