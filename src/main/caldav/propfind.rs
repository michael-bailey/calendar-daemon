/// General `PROPFIND` dispatcher.
///
/// WebDAV clients send `PROPFIND` to discover properties on any resource.
/// This module handles the two remaining cases not covered by `discovery`:
///
/// - `PROPFIND /calendars/{username}/{calendar_id}/` — single calendar props
/// - `PROPFIND /calendars/{username}/{calendar_id}/{uid}.ics` — object props
///
/// The `Depth` header controls recursion: `0` = this resource only,
/// `1` = this resource + immediate children, `infinity` = full tree
/// (we refuse infinity per RFC 4918 §9.1).
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::info;
use crate::repositories::ObjectStore;
use super::xml::{calendar_resourcetype, multistatus, object_resourcetype, prop_response};
use crate::state::app_state::AppState;
use crate::state::error::AppError;

/// `PROPFIND /calendars/{username}/{calendar_id}/`
pub async fn propfind_calendar(
    State(state): State<AppState>,
    Path((username, calendar_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    info!("propfind calendar");
    let depth = depth_header(&headers);

    let calendar = state
        .store
        .get_calendar(&calendar_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let cal_href = format!("/calendars/{username}/{calendar_id}/");
    let color_prop = calendar
        .color
        .as_deref()
        .map(|c| format!("<A:calendar-color>{c}</A:calendar-color>"))
        .unwrap_or_default();

    let cal_props = format!(
        r#"        {resourcetype}
        <D:displayname>{name}</D:displayname>
        {color}
        <D:sync-token>/sync/{calendar_id}/0</D:sync-token>
        <D:current-user-privilege-set>
          <D:privilege><D:read/></D:privilege>
          <D:privilege><D:write/></D:privilege>
        </D:current-user-privilege-set>"#,
        resourcetype = calendar_resourcetype(),
        name = calendar.display_name,
        color = color_prop,
    );

    let mut responses = prop_response(&cal_href, cal_props);

    // Depth: 1 — also include child objects
    if depth == 1 {
        let objects = state.object_store.list_objects(&calendar_id).await?;
        for obj in &objects {
            let obj_href = format!("/calendars/{username}/{calendar_id}/{}.ics", obj.uid);
            let obj_props = format!(
                r#"        {resourcetype}
        <D:getetag>{etag}</D:getetag>
        <D:getcontenttype>text/calendar; charset=utf-8</D:getcontenttype>"#,
                resourcetype = object_resourcetype(),
                etag = obj.etag,
            );
            responses.push_str(&prop_response(&obj_href, obj_props));
        }
    }

    let body = multistatus(responses);
    Ok(xml_207(body))
}

/// `PROPFIND /calendars/{username}/{calendar_id}/{uid}.ics`
pub async fn propfind_object(
    State(state): State<AppState>,
    Path((username, calendar_id, uid)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    info!("propfind object");
    let uid = uid.strip_suffix(".ics").unwrap_or(&uid).to_string();

    let object = state
        .object_store
        .get_object(&calendar_id, &uid)
        .await?
        .ok_or(AppError::NotFound)?;

    let obj_href = format!("/calendars/{username}/{calendar_id}/{uid}.ics");
    let props = format!(
        r#"        {resourcetype}
        <D:getetag>{etag}</D:getetag>
        <D:getcontenttype>text/calendar; charset=utf-8</D:getcontenttype>
        <D:getcontentlength>{len}</D:getcontentlength>"#,
        resourcetype = object_resourcetype(),
        etag = object.etag,
        len = object.ical_data.len(),
    );

    let body = multistatus(prop_response(&obj_href, props));
    Ok(xml_207(body))
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn depth_header(headers: &HeaderMap) -> u8 {
    headers
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn xml_207(body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    (StatusCode::MULTI_STATUS, headers, body).into_response()
}
