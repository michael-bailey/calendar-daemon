use crate::state::error::StoreError;
use async_trait::async_trait;
use calendarkit::domain::{Calendar, CalendarObject};

/// The persistence contract for calendars and their objects.
///
/// All methods are async. Implement this trait to swap between SQLite
/// (production), in-memory (tests), or any future backend.
#[async_trait]
pub trait CalendarStore: Send + Sync {
    // ── Calendars ────────────────────────────────────────────────────────

    /// Return all calendars for the single user.
    async fn list_calendars(&self) -> Result<Vec<Calendar>, StoreError>;

    /// Fetch a single calendar by its ID, or `None` if not found.
    async fn get_calendar(&self, id: &str) -> Result<Option<Calendar>, StoreError>;

    /// Insert or fully replace a calendar.
    async fn upsert_calendar(&self, calendar: Calendar) -> Result<(), StoreError>;

    /// Delete a calendar and all its objects.
    async fn delete_calendar(&self, id: &str) -> Result<(), StoreError>;

    // ── Calendar objects (events / todos) ─────────────────────────────────

    /// Return all objects in a calendar.
    async fn list_objects(&self, calendar_id: &str) -> Result<Vec<CalendarObject>, StoreError>;

    /// Fetch a single object by UID within a calendar.
    async fn get_object(
        &self,
        calendar_id: &str,
        uid: &str,
    ) -> Result<Option<CalendarObject>, StoreError>;

    /// Insert or fully replace a calendar object.
    async fn upsert_object(&self, object: CalendarObject) -> Result<(), StoreError>;

    /// Delete a calendar object by UID.
    async fn delete_object(&self, calendar_id: &str, uid: &str) -> Result<(), StoreError>;

    /// Return just the ETag for an object, without fetching the full ical data.
    /// Useful for cheap conditional-request checks.
    async fn get_object_etag(
        &self,
        calendar_id: &str,
        uid: &str,
    ) -> Result<Option<String>, StoreError>;
}
