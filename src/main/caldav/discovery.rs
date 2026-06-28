use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::info;
use super::xml::{multistatus, principal_resourcetype, prop_response};
use crate::state::app_state::AppState;

/// `GET /.well-known/caldav`
///
/// Redirects Apple Calendar to the context root so it can begin
/// principal discovery. Must be a `301` permanent redirect.
pub async fn well_known_caldav(State(state): State<AppState>) -> Response {
    info!("Well known requested");
    let target = format!("{}/", state.base_url);
    (StatusCode::MOVED_PERMANENTLY, [("Location", target)], "").into_response()
}

/// `PROPFIND /`  (context root)
///
/// Apple Calendar issues a `PROPFIND` here to find `current-user-principal`.
pub async fn propfind_root(State(state): State<AppState>) -> Response {
    info!("propfind root requested");
    let username = &state.principal.username;
    let principal_href = format!("/principals/{username}/");

    let props = format!(
        r#"        <D:current-user-principal>
          <D:href>{principal_href}</D:href>
        </D:current-user-principal>"#
    );

    let body = multistatus(prop_response("/", props));
    xml_207(body)
}

/// `PROPFIND /principals/{username}/`
///
/// Returns the `calendar-home-set` so Apple Calendar knows where
/// to look for calendars.
pub async fn propfind_principal(State(state): State<AppState>) -> Response {
    info!("propfind principal requested");
    let username = &state.principal.username;
    let display_name = &state.principal.display_name;
    let principal_href = format!("/principals/{username}/");
    let home_set_href = format!("/calendars/{username}/");

    let props = format!(
        r#"        {resourcetype}
        <D:displayname>{display_name}</D:displayname>
        <D:principal-URL><D:href>{principal_href}</D:href></D:principal-URL>
        <C:calendar-home-set><D:href>{home_set_href}</D:href></C:calendar-home-set>
        <D:current-user-principal><D:href>{principal_href}</D:href></D:current-user-principal>
        <D:current-user-privilege-set>
          <D:privilege><D:read/></D:privilege>
          <D:privilege><D:write/></D:privilege>
          <D:privilege><D:write-content/></D:privilege>
          <D:privilege><D:bind/></D:privilege>
          <D:privilege><D:unbind/></D:privilege>
        </D:current-user-privilege-set>"#,
        resourcetype = principal_resourcetype(),
        display_name = display_name,
        principal_href = principal_href,
        home_set_href = home_set_href,
    );

    let body = multistatus(prop_response(&principal_href, props));
    xml_207(body)
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn xml_207(body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    (StatusCode::MULTI_STATUS, headers, body).into_response()
}
