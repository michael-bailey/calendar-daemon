use async_trait::async_trait;
use sqlx::Row;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use calendarkit::domain::{Calendar, CalendarObject};
use crate::state::error::StoreError;

pub mod calendar_repository;
pub mod object_repository;

#[async_trait]
pub trait CalendarStore: Send + Sync {
	/// Return all calendars for the single user.
	async fn list_calendars(&self) -> Result<Vec<Calendar>, StoreError>;

	/// Fetch a single calendar by its ID, or `None` if not found.
	async fn get_calendar(&self, id: &str) -> Result<Option<Calendar>, StoreError>;

	/// Insert or fully replace a calendar.
	async fn upsert_calendar(&self, calendar: Calendar) -> Result<(), StoreError>;

	/// Delete a calendar and all its objects.
	async fn delete_calendar(&self, id: &str) -> Result<(), StoreError>;
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
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

trait ObjectHelpers {
	fn calendar_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Calendar, StoreError> {
		Ok(Calendar {
			id: row.try_get("id").map_err(StoreError::Db)?,
			display_name: row.try_get("display_name").map_err(StoreError::Db)?,
			color: row.try_get("color").map_err(StoreError::Db)?,
			description: row.try_get("description").map_err(StoreError::Db)?,
			created_at: Self::parse_time(row.try_get("created_at").map_err(StoreError::Db)?)?,
			updated_at: Self::parse_time(row.try_get("updated_at").map_err(StoreError::Db)?)?,
		})
	}

	fn object_from_row(row: sqlx::sqlite::SqliteRow) -> Result<CalendarObject, StoreError> {
		Ok(CalendarObject {
			uid: row.try_get("uid").map_err(StoreError::Db)?,
			calendar_id: row.try_get("calendar_id").map_err(StoreError::Db)?,
			ical_data: row.try_get("ical_data").map_err(StoreError::Db)?,
			etag: row.try_get("etag").map_err(StoreError::Db)?,
			created_at: Self::parse_time(row.try_get("created_at").map_err(StoreError::Db)?)?,
			updated_at: Self::parse_time(row.try_get("updated_at").map_err(StoreError::Db)?)?,
		})
	}

	fn format_time(value: OffsetDateTime) -> Result<String, StoreError> {
		value
				.format(&Rfc3339)
				.map_err(|e| StoreError::Codec(e.to_string()))
	}

	fn parse_time(value: String) -> Result<OffsetDateTime, StoreError> {
		OffsetDateTime::parse(&value, &Rfc3339).map_err(|e| StoreError::Codec(e.to_string()))
	}
}