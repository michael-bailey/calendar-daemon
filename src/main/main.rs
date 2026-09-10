mod caldav;
pub mod state;
mod store;
pub mod cli;
pub mod database;
pub mod config;
pub mod repositories;
pub mod discovery;
pub mod application;

use std::{str::FromStr, sync::Arc};
use std::net::SocketAddr;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use sqlx::sqlite::SqliteConnectOptions;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter
};

use calendarkit::domain::{Calendar, CalendarObject, Principal};
use config::Config;
use crate::{
    cli::{Cli, Command},
    state::{
        app_state::AppState,
        router
    },
    store::{
        sqlite::SqliteStore,
    }
};
use crate::application::Application;
use crate::database::setup_database;
use crate::repositories::calendar_repository::CalendarRepository;
use crate::repositories::CalendarStore;
use crate::repositories::object_repository::ObjectRepository;
use crate::state::error::StoreError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing — respects RUST_LOG env var, defaults to info
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "calendard=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("loading config");
    let config = Config::from_env_cli()?;

    info!("setting up sqlite storage");
    let sqlite_store = setup_database(&config).await?;

    sqlx::migrate!("./migrations")
        .run(&sqlite_store.pool)
        .await
        .map_err(|e| StoreError::Migration(e.to_string()))
        .expect("Migration failed");

    info!("setting up repositories");
    let calendar_repo = CalendarRepository::new(&sqlite_store.pool).await;
    let object_repo = ObjectRepository::new(&sqlite_store.pool).await;

    info!("creating principal");
    let principal = Principal::new(
        &config.username,
        &config.display_name,
        &config.password_hash,
    );

    info!("creating app state");
    let state = AppState::new(
        calendar_repo.clone(), calendar_repo, object_repo, principal, &config.get_base_url());

    let tls_config = RustlsConfig::from_pem_file(
        &config.tls_cert_path,   // path to your cert.pem
        &config.tls_key_path,    // path to your cert-key.pem
    ).await?;

    let app = Application::new(state);

    app.run_tls(tls_config).await
}

async fn ensure_default_calendar(store: &Arc<SqliteStore>) -> anyhow::Result<()> {
    if store.list_calendars().await?.is_empty() {
        store
            .upsert_calendar(Calendar::new("personal", "Personal"))
            .await?;
    }
    Ok(())
}

fn sqlite_connect_options(database_url: &str) -> anyhow::Result<SqliteConnectOptions> {
    let connection_string = if database_url.starts_with("sqlite:") {
        database_url.to_string()
    } else {
        format!("sqlite:{database_url}")
    };

    Ok(SqliteConnectOptions::from_str(&connection_string)?.create_if_missing(true))
}


