use crate::state::app_state::AppState;
use crate::state::error::AppError;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use calendarkit::domain::CalendarObject;
use crate::repositories::ObjectStore;

/// `GET /calendars/{username}/{calendar_id}/{uid}.ics`
///
/// Returns the raw iCalendar data for a single object.
pub async fn get_object(
    State(state): State<AppState>,
    Path((_username, calendar_id, uid)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    // Strip the .ics suffix if present
    let uid = uid.strip_suffix(".ics").unwrap_or(&uid).to_string();

    let object = state
        .object_store
        .get_object(&calendar_id, &uid)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/calendar; charset=utf-8".parse().unwrap(),
    );
    headers.insert(header::ETAG, object.etag.parse().unwrap());

    Ok((StatusCode::OK, headers, object.ical_data).into_response())
}

/// `PUT /calendars/{username}/{calendar_id}/{uid}.ics`
///
/// Create or update a calendar object.
///
/// Handles `If-Match` (update guard) and `If-None-Match: *` (create guard)
/// conditional headers as required by RFC 4918 §14.
pub async fn put_object(
    State(state): State<AppState>,
    Path((_username, calendar_id, uid)): Path<(String, String, String)>,
    headers: HeaderMap,
    body: String,
) -> Result<Response, AppError> {
    let uid = uid.strip_suffix(".ics").unwrap_or(&uid).to_string();

    let existing_etag = state.object_store.get_object_etag(&calendar_id, &uid).await?;

    // If-None-Match: * means "only create, don't update"
    if let Some(inm) = headers.get(header::IF_NONE_MATCH) {
        if inm.as_bytes() == b"*" && existing_etag.is_some() {
            return Err(AppError::PreconditionFailed);
        }
    }

    // If-Match: "<etag>" means "only update if ETag matches"
    if let Some(im) = headers.get(header::IF_MATCH) {
        match &existing_etag {
            None => return Err(AppError::PreconditionFailed),
            Some(etag) if im.to_str().unwrap_or("") != etag => {
                return Err(AppError::PreconditionFailed);
            }
            _ => {}
        }
    }

    let is_new = existing_etag.is_none();
    let object = CalendarObject::new(&uid, &calendar_id, body);
    state.object_store.upsert_object(object.clone()).await?;

    let status = if is_new {
        StatusCode::CREATED
    } else {
        StatusCode::NO_CONTENT
    };

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::ETAG, object.etag.parse().unwrap());

    Ok((status, resp_headers).into_response())
}

/// `DELETE /calendars/{username}/{calendar_id}/{uid}.ics`
///
/// Delete a calendar object. Supports `If-Match` guard.
pub async fn delete_object(
    State(state): State<AppState>,
    Path((_username, calendar_id, uid)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let uid = uid.strip_suffix(".ics").unwrap_or(&uid).to_string();

    // If-Match guard
    if let Some(im) = headers.get(header::IF_MATCH) {
        let existing = state
            .object_store
            .get_object_etag(&calendar_id, &uid)
            .await?
            .ok_or(AppError::NotFound)?;
        if im.to_str().unwrap_or("") != existing {
            return Err(AppError::PreconditionFailed);
        }
    }

    state.object_store.delete_object(&calendar_id, &uid).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `REPORT /calendars/{username}/{calendar_id}/`
///
/// Handles `calendar-query` and `calendar-multiget` REPORT types.
/// Apple Calendar uses these to fetch events in a date range and to
/// sync specific UIDs after a change.
///
/// Stub — full XML parsing of the request body comes next.
pub async fn report_calendar(
    State(state): State<AppState>,
    Path((username, calendar_id)): Path<(String, String)>,
    _body: String,
) -> Result<Response, AppError> {
    // TODO: parse report type from body (calendar-query vs calendar-multiget)
    // For now return all objects in the calendar — good enough for initial
    // Apple Calendar handshake testing.
    let objects = state.object_store.list_objects(&calendar_id).await?;
    let responses = build_object_responses(&username, &calendar_id, &objects);

    let body = super::xml::multistatus(responses);
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    Ok((StatusCode::MULTI_STATUS, headers, body).into_response())
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn build_object_responses(username: &str, calendar_id: &str, objects: &[CalendarObject]) -> String {
    objects
        .iter()
        .map(|obj| {
            let href = format!("/calendars/{username}/{calendar_id}/{}.ics", obj.uid);
            let props = format!(
                r#"        <D:getetag>{etag}</D:getetag>
        <C:calendar-data>{ical}</C:calendar-data>"#,
                etag = obj.etag,
                ical = escape_xml(&obj.ical_data),
            );
            super::xml::prop_response(&href, props)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::escape_xml;

    #[test]
    fn test_xml_escape() {
        assert_eq!(escape_xml("a & b < c > d"), "a &amp; b &lt; c &gt; d");
        assert_eq!(escape_xml("no special chars"), "no special chars");
    }
}
