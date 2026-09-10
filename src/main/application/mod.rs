use std::io;
use axum::{middleware, Router, ServiceExt};
use std::net::{SocketAddr, TcpListener};
use axum_server::Address;
use axum_server::tls_rustls::RustlsConfig;
use tower_http::trace::TraceLayer;
use tracing::info;
use crate::state::app_state::AppState;
use crate::state::router::build_router;

pub struct Application(pub Router);

impl Application {
	pub fn new(app_state: AppState) -> Application {
		info!("Created application router");
		let mut router = Router::new();

		info!("Merging discovery routes");
		router = router.merge(crate::discovery::setup_routes());

		info!("Adding tracing layer");
		router = router
				.layer(TraceLayer::new_for_http());

		info!("Adding app state");
		let router = router.with_state(app_state);
		Application(router)
	}

	pub async fn run(self) -> io::Result<()> {
		let address: SocketAddr = "0.0.0.0:8000".parse().expect("invalid IP address");
		let server = axum_server::bind(address);
		let service = self.0.into_make_service();
		server.serve(service).await
	}

	pub async fn run_tls(self, tls: RustlsConfig) -> anyhow::Result<()> {
		let address: SocketAddr = "0.0.0.0:8000".parse().expect("invalid IP address");
		let server = axum_server::bind_rustls(address, tls);
		let service = self.0.into_make_service();
		server.serve(service).await?;
		Ok(())
	}
}