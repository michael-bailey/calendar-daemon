use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::STANDARD, Engine};

use crate::state::app_state::AppState;

/// Axum middleware that enforces HTTP Basic authentication.
///
/// On failure it returns `401` with a `WWW-Authenticate` challenge so
/// that Apple Calendar and iOS prompt for credentials.
pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, [(header::HeaderName, &'static str); 1])> {
    let unauthorized = || {
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"CalDAV\"")],
        )
    };

    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(unauthorized)?;

    let credentials = auth_header
        .strip_prefix("Basic ")
        .and_then(|b64| STANDARD.decode(b64).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Basic realm=\"CalDAV\"")],
        ))?;

    let (username, password) = credentials.split_once(':').ok_or((
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic realm=\"CalDAV\"")],
    ))?;

    if username != state.principal.username {
        return Err(unauthorized());
    }

    let valid_password = bcrypt::verify(password, &state.principal.password_hash).unwrap_or(false);
    if !valid_password {
        return Err(unauthorized());
    }

    Ok(next.run(request).await)
}
