use time::OffsetDateTime;

/// A single calendar object resource — one `.ics` file, containing a
/// VEVENT, VTODO, or VJOURNAL component.
#[derive(Debug, Clone)]
pub struct CalendarObject {
    /// Stable UID — matches the UID inside the iCalendar data
    pub uid: String,
    /// The calendar this object belongs to
    pub calendar_id: String,
    /// Raw iCalendar data (the full VCALENDAR block)
    pub ical_data: String,
    /// ETag for conditional requests (If-Match / If-None-Match)
    ///
    /// Derived from a hash of `ical_data`; clients use this to detect
    /// concurrent modifications.
    pub etag: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl CalendarObject {
    pub fn new(
        uid: impl Into<String>,
        calendar_id: impl Into<String>,
        ical_data: impl Into<String>,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        let data: String = ical_data.into();
        let etag = compute_etag(&data);
        Self {
            uid: uid.into(),
            calendar_id: calendar_id.into(),
            ical_data: data,
            etag,
            created_at: now,
            updated_at: now,
        }
    }

    /// Recompute the ETag after the ical data changes.
    pub fn refresh_etag(&mut self) {
        self.etag = compute_etag(&self.ical_data);
        self.updated_at = OffsetDateTime::now_utc();
    }
}

/// Simple deterministic ETag based on content hash.
/// Wrapped in quotes as required by RFC 7232.
fn compute_etag(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    format!("\"{}\"", h.finish())
}
