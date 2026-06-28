use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use calendarkit::domain::{Calendar, CalendarObject};
use crate::repositories::{ObjectHelpers, ObjectStore};
use crate::repositories::calendar_repository::CalendarRepository;
use crate::state::error::StoreError;

pub struct ObjectRepository {
	pool: Pool<Sqlite>,
}

impl ObjectRepository {
	pub async fn new(
		pool: &SqlitePool
	) -> Arc<ObjectRepository> {
		Arc::new(Self {
			pool: pool.clone(),
		})
	}
}

impl ObjectHelpers for ObjectRepository {  }

#[async_trait]
impl ObjectStore for ObjectRepository {
	async fn list_objects(&self, calendar_id: &str) -> Result<Vec<CalendarObject>, StoreError> {
		let rows = sqlx::query(
			"SELECT uid, calendar_id, ical_data, etag, created_at, updated_at
             FROM calendar_objects WHERE calendar_id = ?",
		)
				.bind(calendar_id)
				.fetch_all(&self.pool)
				.await
				.map_err(StoreError::Db)?;

		rows.into_iter().map(Self::object_from_row).collect()
	}

	async fn get_object(
		&self,
		calendar_id: &str,
		uid: &str,
	) -> Result<Option<CalendarObject>, StoreError> {
		let row = sqlx::query(
			"SELECT uid, calendar_id, ical_data, etag, created_at, updated_at
             FROM calendar_objects WHERE calendar_id = ? AND uid = ?",
		)
				.bind(calendar_id)
				.bind(uid)
				.fetch_optional(&self.pool)
				.await
				.map_err(StoreError::Db)?;

		row.map(Self::object_from_row).transpose()
	}

	async fn upsert_object(&self, object: CalendarObject) -> Result<(), StoreError> {
		sqlx::query(
			"INSERT INTO calendar_objects (uid, calendar_id, ical_data, etag, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(uid, calendar_id) DO UPDATE SET
               ical_data  = excluded.ical_data,
               etag       = excluded.etag,
               updated_at = excluded.updated_at",
		)
				.bind(object.uid)
				.bind(object.calendar_id)
				.bind(object.ical_data)
				.bind(object.etag)
				.bind(Self::format_time(object.created_at)?)
				.bind(Self::format_time(object.updated_at)?)
				.execute(&self.pool)
				.await
				.map_err(StoreError::Db)?;
		Ok(())
	}

	async fn delete_object(&self, calendar_id: &str, uid: &str) -> Result<(), StoreError> {
		sqlx::query("DELETE FROM calendar_objects WHERE calendar_id = ? AND uid = ?")
				.bind(calendar_id)
				.bind(uid)
				.execute(&self.pool)
				.await
				.map_err(StoreError::Db)?;
		Ok(())
	}

	async fn get_object_etag(
		&self,
		calendar_id: &str,
		uid: &str,
	) -> Result<Option<String>, StoreError> {
		sqlx::query_scalar("SELECT etag FROM calendar_objects WHERE calendar_id = ? AND uid = ?")
				.bind(calendar_id)
				.bind(uid)
				.fetch_optional(&self.pool)
				.await
				.map_err(StoreError::Db)
	}
}