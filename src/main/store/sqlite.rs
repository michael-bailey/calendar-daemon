use crate::state::error::StoreError;
use async_trait::async_trait;
use calendarkit::domain::{Calendar, CalendarObject};
use sqlx::{Row, SqlitePool};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use crate::repositories::{CalendarStore, ObjectStore};

/// SQLite-backed `CalendarStore`.
///
/// Initialise with [`SqliteStore::new`], which runs migrations automatically.
pub struct SqliteStore {
    pub pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(pool: SqlitePool) -> Result<Self, StoreError> {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| StoreError::Migration(e.to_string()))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl CalendarStore for SqliteStore {
    async fn list_calendars(&self) -> Result<Vec<Calendar>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, display_name, color, description, created_at, updated_at FROM calendars",
        )
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::Db)?;

        rows.into_iter().map(calendar_from_row).collect()
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

        row.map(calendar_from_row).transpose()
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
            .bind(format_time(calendar.created_at)?)
            .bind(format_time(calendar.updated_at)?)
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

#[async_trait]
impl ObjectStore for SqliteStore {
    async fn list_objects(&self, calendar_id: &str) -> Result<Vec<CalendarObject>, StoreError> {
        let rows = sqlx::query(
            "SELECT uid, calendar_id, ical_data, etag, created_at, updated_at
             FROM calendar_objects WHERE calendar_id = ?",
        )
        .bind(calendar_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::Db)?;

        rows.into_iter().map(object_from_row).collect()
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

        row.map(object_from_row).transpose()
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
        .bind(format_time(object.created_at)?)
        .bind(format_time(object.updated_at)?)
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


fn calendar_from_row(row: sqlx::sqlite::SqliteRow) -> Result<Calendar, StoreError> {
    Ok(Calendar {
        id: row.try_get("id").map_err(StoreError::Db)?,
        display_name: row.try_get("display_name").map_err(StoreError::Db)?,
        color: row.try_get("color").map_err(StoreError::Db)?,
        description: row.try_get("description").map_err(StoreError::Db)?,
        created_at: parse_time(row.try_get("created_at").map_err(StoreError::Db)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(StoreError::Db)?)?,
    })
}

fn object_from_row(row: sqlx::sqlite::SqliteRow) -> Result<CalendarObject, StoreError> {
    Ok(CalendarObject {
        uid: row.try_get("uid").map_err(StoreError::Db)?,
        calendar_id: row.try_get("calendar_id").map_err(StoreError::Db)?,
        ical_data: row.try_get("ical_data").map_err(StoreError::Db)?,
        etag: row.try_get("etag").map_err(StoreError::Db)?,
        created_at: parse_time(row.try_get("created_at").map_err(StoreError::Db)?)?,
        updated_at: parse_time(row.try_get("updated_at").map_err(StoreError::Db)?)?,
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
