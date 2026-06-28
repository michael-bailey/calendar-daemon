use super::xml::{calendar_resourcetype, multistatus, prop_response};
use crate::state::app_state::AppState;
use crate::state::error::AppError;
/// Handlers for calendar collection operations.
///
/// These handle `PROPFIND /calendars/{user}/` and
/// `PROPFIND /calendars/{user}/{calendar_id}/`.
///
/// Stubs — implementation comes once discovery is verified working.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::info;
use calendarkit::domain::Calendar;
use crate::caldav::xml;

/// `PROPFIND /calendars/{username}/`
///
/// Lists all calendars for the user.
pub async fn propfind_home_set(State(state): State<AppState>) -> Result<Response, AppError> {
    info!("Starting propfind home set");
    let username = &state.principal.username;
    let home_href = format!("/calendars/{username}/");

    let calendars = state.store.list_calendars().await?;

    let mut responses = prop_response(
        &home_href,
        format!(
            r#"        <D:resourcetype><D:collection/></D:resourcetype>
        <D:displayname>Calendars</D:displayname>
        <D:current-user-principal>
          <D:href>/principals/{username}/</D:href>
        </D:current-user-principal>"#
        ),
    );

    for cal in &calendars {
        let cal_href = format!("/calendars/{username}/{}/", cal.id);
        let color_prop = cal
            .color
            .as_deref()
            .map(|c| format!("<A:calendar-color>{c}</A:calendar-color>"))
            .unwrap_or_default();

        let props = format!(
            r#"        {resourcetype}
        <D:displayname>{name}</D:displayname>
        {color}
        <D:current-user-privilege-set>
          <D:privilege><D:read/></D:privilege>
          <D:privilege><D:write/></D:privilege>
        </D:current-user-privilege-set>"#,
            resourcetype = calendar_resourcetype(),
            name = cal.display_name,
            color = color_prop,
        );

        responses.push_str(&prop_response(&cal_href, props));
    }

    let body = multistatus(responses);
    Ok(xml_207(body))
}

/// `MKCALENDAR /calendars/{username}/{calendar_id}/`
///
/// Creates a calendar collection. Apple Calendar uses this when the user
/// creates a new calendar on the account.
pub async fn mkcalendar(
    State(state): State<AppState>,
    Path((_username, calendar_id)): Path<(String, String)>,
    body: String,
) -> Result<Response, AppError> {

    info!("creating calendar");

    if state.store.get_calendar(&calendar_id).await?.is_some() {
        return Err(AppError::PreconditionFailed);
    }

    let display_name = extract_display_name(&body).unwrap_or_else(|| calendar_id.clone());
    state
        .store
        .upsert_calendar(Calendar::new(calendar_id, display_name))
        .await?;

    Ok(StatusCode::CREATED.into_response())
}

fn extract_display_name(body: &str) -> Option<String> {
    let start_tag = "<D:displayname>";
    let end_tag = "</D:displayname>";
    let start = body.find(start_tag)? + start_tag.len();
    let end = body[start..].find(end_tag)? + start;
    Some(unescape_xml(body[start..end].trim()))
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn xml_207(body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    (StatusCode::MULTI_STATUS, headers, body).into_response()
}

pub async fn proppatch_calendar(
    State(state): State<AppState>,
    Path((username, calendar_id)): Path<(String, String)>,
    body: String,
) -> Result<Response, AppError> {
    // Parse which properties were set so we can echo them back.
    // macOS Calendar sends default-alarm-vevent-date etc — we acknowledge
    // without storing, which is sufficient to keep it happy.
    let href = format!("/calendars/{username}/{calendar_id}/");

    // Minimal acknowledgement — echo each prop name back with 200 OK.
    // A real implementation would parse the XML and reflect each property.
    let props = r#"        <D:displayname/>"#;
    let body = xml::multistatus(xml::prop_response(&href, props));

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/xml; charset=utf-8".parse().unwrap());
    Ok((StatusCode::MULTI_STATUS, headers, body).into_response())
}