mod caldav;
pub mod state;
mod store;
pub mod cli;
pub mod database;
pub mod config;
pub mod repositories;

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

    match Cli::parse().command {
        None => run_server().await,
        Some(Command::Passwd { password }) => {
            println!("{}", bcrypt::hash(password, bcrypt::DEFAULT_COST)?);
            Ok(())
        }
    }
}

async fn run_server() -> anyhow::Result<()> {
    info!("Starting server");

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

    state.store.upsert_calendar(Calendar::new("cal", "Main Calendar")).await?;

    info!("creating router");
    let app: Router = router::build_router(state)
        .layer(TraceLayer::new_for_http());

    info!("CalDAV server listening on {}", config.get_bind_address());
    info!("Base URL: {}", config.get_base_url());

    let tls_config = RustlsConfig::from_pem_file(
        &config.tls_cert_path,   // path to your cert.pem
        &config.tls_key_path,    // path to your cert-key.pem
    ).await?;

    let address: SocketAddr = config.get_bind_address().parse()?;

    let server = axum_server::bind_rustls(address, tls_config);
    server.serve(app.into_make_service()).await?;
    Ok(())
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


