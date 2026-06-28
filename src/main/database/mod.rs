mod database_factory;

use std::sync::Arc;
use tracing::info;
use sqlx::sqlite::SqlitePoolOptions;

use crate::{ensure_default_calendar, sqlite_connect_options};
use crate::config::Config;
use crate::store::sqlite::SqliteStore;

pub async fn setup_database(
	config: &Config,
) -> anyhow::Result<Arc<SqliteStore>> {
	prepare_sqlite_parent_dir(&config.database_url)?;
	let database_options = sqlite_connect_options(&config.database_url)?;

	// Database pool
	let pool = SqlitePoolOptions::new()
			.max_connections(5)
			.connect_with(database_options)
			.await?;

	info!("Database connection established");

	let store = Arc::new(SqliteStore::new(pool).await?);
	ensure_default_calendar(&store).await?;
	Ok(store)
}

pub fn prepare_sqlite_parent_dir(database_url: &str) -> anyhow::Result<()> {
	let path_and_query = database_url.strip_prefix("sqlite:").unwrap_or(database_url);

	if path_and_query.starts_with(":memory:") {
		return Ok(());
	}

	let path = path_and_query
			.strip_prefix("//")
			.unwrap_or(path_and_query)
			.split_once('?')
			.map(|(path, _)| path)
			.unwrap_or(path_and_query);

	if path.is_empty() {
		return Ok(());
	}

	let path = std::path::Path::new(path);

	if path
			.parent()
			.is_some_and(|parent| parent.as_os_str().is_empty())
	{
		return Ok(());
	}

	if let Some(parent) = path.parent() {
		if !parent.as_os_str().is_empty() {
			std::fs::create_dir_all(parent)?;
		}
	}

	Ok(())
}