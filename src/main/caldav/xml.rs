/// Minimal XML builders for WebDAV/CalDAV responses.
///
/// These produce the `207 Multi-Status` and `PROPFIND` response bodies
/// that Apple Calendar expects. Using string templating here is
/// intentional — the response shapes are fixed and small, so a full XML
/// DOM library would add complexity without benefit.

/// Wrap a set of `<response>` blocks in a `<multistatus>` envelope.
pub fn multistatus(responses: impl Into<String>) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<D:multistatus xmlns:D="DAV:"
               xmlns:C="urn:ietf:params:xml:ns:caldav"
               xmlns:A="http://apple.com/ns/ical/">
{}
</D:multistatus>"#,
        responses.into()
    )
}

/// Build a single `<response>` block with a `200 OK` propstat.
pub fn prop_response(href: &str, props: impl Into<String>) -> String {
    format!(
        r#"  <D:response>
    <D:href>{href}</D:href>
    <D:propstat>
      <D:prop>
{props}
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>"#,
        href = href,
        props = props.into(),
    )
}

/// The `resourcetype` property for a calendar collection.
pub fn calendar_resourcetype() -> &'static str {
    "<D:resourcetype><D:collection/><C:calendar/></D:resourcetype>"
}

/// The `resourcetype` property for a principal collection.
pub fn principal_resourcetype() -> &'static str {
    "<D:resourcetype><D:collection/><D:principal/></D:resourcetype>"
}

/// The `resourcetype` for a plain calendar object (.ics).
pub fn object_resourcetype() -> &'static str {
    "<D:resourcetype/>"
}
