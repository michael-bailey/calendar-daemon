use axum::response::{IntoResponse, Redirect, Response};

pub async fn handle_well_known() -> Response {
	Redirect::temporary("/cal/").into_response()
}

