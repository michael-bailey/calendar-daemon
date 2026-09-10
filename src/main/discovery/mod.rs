use std::convert::Infallible;
use axum::Router;
use axum::routing::{any, get, MethodRouter};
use crate::discovery::well_known::handle_well_known;
use crate::state::app_state::AppState;

mod well_known;

pub fn setup_routes() -> Router<AppState> {
	Router::new()
			.route("/.well-known/caldav", any(handle_well_known))
}

