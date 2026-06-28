use crate::caldav::{calendar, discovery, object, propfind};
use crate::state::app_state::AppState;
use crate::state::auth::require_auth;
use axum::response::IntoResponse;
use axum::{middleware, routing::get, Router};
use tracing::debug;

/// Build the full application router.
///
/// Route structure:
/// ```text
/// GET  /.well-known/caldav                              → redirect
/// *    /                                                → PROPFIND context root
/// *    /principals/{username}/                          → PROPFIND principal
/// *    /calendars/{username}/                           → PROPFIND home-set
/// *    /calendars/{username}/{calendar_id}/             → PROPFIND calendar / REPORT
/// *    /calendars/{username}/{calendar_id}/{uid}.ics    → GET / PUT / DELETE / PROPFIND object
/// ```
///
/// All routes except `.well-known` are protected by Basic auth middleware.
pub fn build_router(state: AppState) -> Router {
    // Public — discovery redirect only (Apple needs this before auth)
    let public = Router::new()
        .route("/.well-known/caldav", get(discovery::well_known_caldav))
        .route("/.well-known/calendar", get(discovery::well_known_caldav));

    // Protected — everything else requires Basic auth
    let protected = Router::new()
        // Context root
        .route("/", axum::routing::any(propfind_dispatch_root))
        // Principal
        .route(
            "/principals/{username}/",
            axum::routing::any(propfind_dispatch_principal),
        )
        // Calendar home-set
        .route(
            "/calendars/{username}/",
            axum::routing::any(calendar::propfind_home_set),
        )
        // Calendar collection
        .route(
            "/calendars/{username}/{calendar_id}/",
            axum::routing::any(calendar_collection_dispatch),
        )
        // Calendar object
        .route(
            "/calendars/{username}/{calendar_id}/{uid}",
            axum::routing::any(object_dispatch),
        )
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    public.merge(protected).with_state(state)
}

// ── per-method dispatchers ────────────────────────────────────────────────────
//
// axum's `routing::any` catches all HTTP methods. We dispatch on `Method`
// manually because WebDAV introduces non-standard methods (PROPFIND, REPORT,
// MKCALENDAR) that axum has no built-in routing helpers for.

async fn propfind_dispatch_root(
    state: axum::extract::State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    match req.method().as_str() {
        "PROPFIND" | "GET" | "HEAD" => discovery::propfind_root(state).await,
        "OPTIONS" => options_response(),
        _ => method_not_allowed(),
    }
}

async fn propfind_dispatch_principal(
    state: axum::extract::State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    match req.method().as_str() {
        "PROPFIND" => discovery::propfind_principal(state).await,
        "OPTIONS" => options_response(),
        _ => method_not_allowed(),
    }
}

async fn calendar_collection_dispatch(
    state: axum::extract::State<AppState>,
    path: axum::extract::Path<(String, String)>,
    headers: axum::http::HeaderMap,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    debug!("request from {:?}", path);
    debug!("method: {:?}", req.method());
    debug!("headers: {:?}", headers);
    use axum::body::to_bytes;
    let method = req.method().as_str().to_uppercase();
    match method.as_str() {
        "PROPFIND" => propfind::propfind_calendar(state, path, headers)
            .await
            .unwrap_or_else(|e| e.into_response()),
        "PROPPATCH" => {
            let (_, body) = req.into_parts();
            let bytes = to_bytes(body, 1024 * 64).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&bytes).to_string();
            calendar::proppatch_calendar(state, path, body_str)
                .await
                .unwrap_or_else(|e| e.into_response())
        }
        "REPORT" | "GET" => {
            let (_, body) = req.into_parts();
            let bytes = to_bytes(body, 1024 * 64).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&bytes).to_string();
            object::report_calendar(state, path, body_str)
                .await
                .unwrap_or_else(|e| e.into_response())
        }
        "MKCALENDAR" => {
            let (_, body) = req.into_parts();
            let bytes = to_bytes(body, 1024 * 64).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&bytes).to_string();
            calendar::mkcalendar(state, path, body_str)
                .await
                .unwrap_or_else(|e| e.into_response())
        }
        "OPTIONS" => options_response(),
        _ => method_not_allowed(),
    }
}

async fn object_dispatch(
    state: axum::extract::State<AppState>,
    path: axum::extract::Path<(String, String, String)>,
    headers: axum::http::HeaderMap,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    use axum::body::to_bytes;
    let method = req.method().as_str().to_uppercase();
    match method.as_str() {
        "GET" | "HEAD" => object::get_object(state, path)
            .await
            .unwrap_or_else(|e| e.into_response()),
        "PUT" => {
            let (_, body) = req.into_parts();
            let bytes = to_bytes(body, 1024 * 512).await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&bytes).to_string();
            object::put_object(state, path, headers, body_str)
                .await
                .unwrap_or_else(|e| e.into_response())
        }
        "DELETE" => object::delete_object(state, path, headers)
            .await
            .unwrap_or_else(|e| e.into_response()),
        "PROPFIND" => propfind::propfind_object(state, path)
            .await
            .unwrap_or_else(|e| e.into_response()),
        "OPTIONS" => options_response(),
        _ => method_not_allowed(),
    }
}

fn options_response() -> axum::response::Response {
    use axum::http::StatusCode;
    (
        StatusCode::NO_CONTENT,
        [
            ("DAV", "1, 2, access-control, calendar-access"),
            (
                "Allow",
                "OPTIONS, GET, HEAD, PUT, DELETE, PROPFIND, PROPPATCH, REPORT, MKCALENDAR"
            ),
        ],
    )
        .into_response()
}

fn method_not_allowed() -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    StatusCode::METHOD_NOT_ALLOWED.into_response()
}
