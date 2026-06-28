use std::sync::Arc;

use async_trait::async_trait;
use calendarkit::domain::Calendar;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::{
	state::error::StoreError,
	repositories::{CalendarStore, ObjectHelpers}
};

pub struct CalendarRepository {
	pool: Pool<Sqlite>,
}

impl CalendarRepository {
	pub async fn new(
		pool: &SqlitePool
	) -> Arc<CalendarRepository> {
		Arc::new(Self {
			pool: pool.clone(),
		})
	}
}

impl ObjectHelpers for CalendarRepository {  }

#[async_trait]
impl CalendarStore for CalendarRepository {
	async fn list_calendars(&self) -> Result<Vec<Calendar>, StoreError> {
		let rows = sqlx::query(
			"SELECT id, display_name, color, description, created_at, updated_at FROM calendars",
		)
				.fetch_all(&self.pool)
				.await
				.map_err(StoreError::Db)?;

		rows.into_iter().map(Self::calendar_from_row).collect()
	}

	async fn get_calendar(&self, id: &str) -> Result<Option<Calendar>, StoreError> {
		let row = sqlx::query(
			"SELECT id, display_name, color, description, created_at, updated_at
             FROM calendars WHERE id = ?",
		)
				.bind(id)
				.fetch_optional(&self.pool)
				.await
				.map_err(StoreError::Db)?;

		row.map(Self::calendar_from_row).transpose()
	}

	async fn upsert_calendar(&self, calendar: Calendar) -> Result<(), StoreError> {
		sqlx::query(
			"INSERT INTO calendars (id, display_name, color, description, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               display_name = excluded.display_name,
               color        = excluded.color,
               description  = excluded.description,
               updated_at   = excluded.updated_at",
		)
				.bind(calendar.id)
				.bind(calendar.display_name)
				.bind(calendar.color)
				.bind(calendar.description)
				.bind(Self::format_time(calendar.created_at)?)
				.bind(Self::format_time(calendar.updated_at)?)
				.execute(&self.pool)
				.await
				.map_err(StoreError::Db)?;
		Ok(())
	}

	async fn delete_calendar(&self, id: &str) -> Result<(), StoreError> {
		sqlx::query("DELETE FROM calendars WHERE id = ?")
				.bind(id)
				.execute(&self.pool)
				.await
				.map_err(StoreError::Db)?;
		Ok(())
	}
}