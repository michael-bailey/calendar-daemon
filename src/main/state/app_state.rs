use crate::store::DynCalendarStore;
use calendarkit::domain::Principal;
use std::sync::Arc;
use crate::repositories::calendar_repository::CalendarRepository;
use crate::repositories::CalendarStore;
use crate::repositories::object_repository::ObjectRepository;

/// Shared application state injected into every axum handler.
/// Clone is cheap — all inner types are `Arc`-backed.
#[derive(Clone)]
pub struct AppState {
    /// The calendar persistence layer. Swap the concrete type for tests.
    pub store: Arc<dyn CalendarStore>,
    /// Repository for calendars
    pub calendar_store: Arc<CalendarRepository>,
    /// Repository for calendar objects
    pub object_store: Arc<ObjectRepository>,
    /// The single authorised principal.
    pub principal: Arc<Principal>,
    /// Base URL of this server, e.g. "https://cal.example.com"
    pub base_url: String,
}

impl AppState {
    pub fn new(
        store: Arc<dyn CalendarStore>,
        calendar_store: Arc<CalendarRepository>,
        object_store: Arc<ObjectRepository>,
        principal: Principal,
        base_url: impl Into<String>
    ) -> Self {
        Self {
            store,
            calendar_store,
            object_store,
            principal: Arc::new(principal),
            base_url: base_url.into(),
        }
    }
}
