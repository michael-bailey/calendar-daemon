pub mod calendar_store;
pub mod sqlite;

pub use calendar_store::CalendarStore;

use std::sync::Arc;

/// Convenience type alias used throughout the app.
pub type DynCalendarStore = Arc<dyn CalendarStore>;
