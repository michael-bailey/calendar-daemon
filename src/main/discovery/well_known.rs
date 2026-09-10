use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use crate::state::app_state::AppState;

pub async fn handle_well_known(request: Request<AppState>) -> Response {
	let is_propfind = request.method() == "PROFIND";

	if is_propfind { Redirect::temporary("/cal/").into_response() }
	else { StatusCode::METHOD_NOT_ALLOWED.into_response() }

}

