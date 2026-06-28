-- Migration 0001: initial schema

CREATE TABLE IF NOT EXISTS calendars (
    id            TEXT PRIMARY KEY NOT NULL,
    display_name  TEXT NOT NULL,
    color         TEXT,
    description   TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS calendar_objects (
    uid          TEXT NOT NULL,
    calendar_id  TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    ical_data    TEXT NOT NULL,
    etag         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (uid, calendar_id)
);

CREATE INDEX IF NOT EXISTS idx_objects_calendar_id
    ON calendar_objects(calendar_id);
