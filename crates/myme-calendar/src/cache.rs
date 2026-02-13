//! SQLite-based offline cache for Calendar events.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::types::{AccessRole, Calendar, Event, EventStatus, EventTime};

/// SQLite cache for Calendar data.
pub struct CalendarCache {
    conn: Connection,
}

impl CalendarCache {
    /// Create a new cache at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Create an in-memory cache (for testing).
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS calendars (
                id TEXT PRIMARY KEY,
                summary TEXT NOT NULL,
                description TEXT,
                time_zone TEXT,
                background_color TEXT,
                foreground_color TEXT,
                is_primary INTEGER NOT NULL,
                access_role TEXT NOT NULL,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                id TEXT NOT NULL,
                calendar_id TEXT NOT NULL,
                summary TEXT NOT NULL,
                description TEXT,
                location TEXT,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                all_day INTEGER NOT NULL,
                attendees_json TEXT NOT NULL,
                organizer TEXT,
                status TEXT NOT NULL,
                html_link TEXT,
                etag TEXT,
                cached_at INTEGER NOT NULL,
                PRIMARY KEY (id, calendar_id)
            );

            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_events_calendar ON events(calendar_id);
            CREATE INDEX IF NOT EXISTS idx_events_start ON events(start_ms);
            "#,
        )?;
        Ok(())
    }

    /// Store a calendar in the cache.
    pub fn store_calendar(&self, calendar: &Calendar) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        let access_role = match calendar.access_role {
            AccessRole::Owner => "owner",
            AccessRole::Writer => "writer",
            AccessRole::Reader => "reader",
            AccessRole::FreeBusyReader => "freeBusyReader",
        };

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO calendars
            (id, summary, description, time_zone, background_color, foreground_color, is_primary, access_role, cached_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                calendar.id,
                calendar.summary,
                calendar.description,
                calendar.time_zone,
                calendar.background_color,
                calendar.foreground_color,
                calendar.is_primary as i32,
                access_role,
                now,
            ],
        )?;
        Ok(())
    }

    /// List all calendars from cache.
    pub fn list_calendars(&self) -> Result<Vec<Calendar>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, summary, description, time_zone, background_color, foreground_color, is_primary, access_role FROM calendars"
        )?;

        let rows = stmt.query_map([], |row| {
            let role_str: String = row.get(7)?;
            Ok(Calendar {
                id: row.get(0)?,
                summary: row.get(1)?,
                description: row.get(2)?,
                time_zone: row.get(3)?,
                background_color: row.get(4)?,
                foreground_color: row.get(5)?,
                is_primary: row.get::<_, i32>(6)? != 0,
                access_role: match role_str.as_str() {
                    "owner" => AccessRole::Owner,
                    "writer" => AccessRole::Writer,
                    "freeBusyReader" => AccessRole::FreeBusyReader,
                    _ => AccessRole::Reader,
                },
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read calendars: {}", e))
    }

    /// Store an event in the cache.
    pub fn store_event(&self, event: &Event) -> Result<()> {
        let now = Utc::now().timestamp_millis();
        let attendees_json = serde_json::to_string(&event.attendees)?;
        let status = match event.status {
            EventStatus::Confirmed => "confirmed",
            EventStatus::Tentative => "tentative",
            EventStatus::Cancelled => "cancelled",
        };

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO events
            (id, calendar_id, summary, description, location, start_ms, end_ms, all_day, attendees_json, organizer, status, html_link, etag, cached_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                event.id,
                event.calendar_id,
                event.summary,
                event.description,
                event.location,
                event.start.as_datetime().timestamp_millis(),
                event.end.as_datetime().timestamp_millis(),
                event.all_day as i32,
                attendees_json,
                event.organizer,
                status,
                event.html_link,
                event.etag,
                now,
            ],
        )?;
        Ok(())
    }

    /// Get an event from the cache.
    pub fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<Option<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, calendar_id, summary, description, location, start_ms, end_ms, all_day, attendees_json, organizer, status, html_link, etag FROM events WHERE id = ?1 AND calendar_id = ?2"
        )?;

        let mut rows = stmt.query(params![event_id, calendar_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_event(row)?))
        } else {
            Ok(None)
        }
    }

    /// List events in a time range.
    pub fn list_events(
        &self,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
    ) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, calendar_id, summary, description, location, start_ms, end_ms, all_day, attendees_json, organizer, status, html_link, etag
            FROM events
            WHERE calendar_id = ?1 AND start_ms >= ?2 AND start_ms < ?3
            ORDER BY start_ms ASC
            "#
        )?;

        let rows = stmt.query_map(
            params![calendar_id, time_min.timestamp_millis(), time_max.timestamp_millis()],
            Self::row_to_event,
        )?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read events: {}", e))
    }

    /// Get events for today.
    pub fn get_today_events(&self, calendar_id: &str) -> Result<Vec<Event>> {
        let today = Utc::now().date_naive();
        let start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = today.and_hms_opt(23, 59, 59).unwrap().and_utc();
        self.list_events(calendar_id, start, end)
    }

    /// Delete an event from the cache.
    pub fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM events WHERE id = ?1 AND calendar_id = ?2",
            params![event_id, calendar_id],
        )?;
        Ok(())
    }

    /// Get upcoming events count.
    pub fn upcoming_event_count(&self, calendar_id: &str, hours: i64) -> Result<u32> {
        let now = Utc::now().timestamp_millis();
        let future = (Utc::now() + chrono::Duration::hours(hours)).timestamp_millis();

        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM events WHERE calendar_id = ?1 AND start_ms >= ?2 AND start_ms < ?3",
            params![calendar_id, now, future],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Clear all cached data.
    pub fn clear(&self) -> Result<()> {
        self.conn
            .execute_batch("DELETE FROM events; DELETE FROM calendars; DELETE FROM sync_state;")?;
        Ok(())
    }

    fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
        let start_ms: i64 = row.get(5)?;
        let end_ms: i64 = row.get(6)?;
        let all_day: i32 = row.get(7)?;
        let attendees_json: String = row.get(8)?;
        let status_str: String = row.get(10)?;

        let start = if all_day != 0 {
            EventTime::Date(
                DateTime::from_timestamp_millis(start_ms).unwrap_or_default().date_naive(),
            )
        } else {
            EventTime::DateTime(DateTime::from_timestamp_millis(start_ms).unwrap_or_default())
        };

        let end = if all_day != 0 {
            EventTime::Date(
                DateTime::from_timestamp_millis(end_ms).unwrap_or_default().date_naive(),
            )
        } else {
            EventTime::DateTime(DateTime::from_timestamp_millis(end_ms).unwrap_or_default())
        };

        let status = match status_str.as_str() {
            "tentative" => EventStatus::Tentative,
            "cancelled" => EventStatus::Cancelled,
            _ => EventStatus::Confirmed,
        };

        Ok(Event {
            id: row.get(0)?,
            calendar_id: row.get(1)?,
            summary: row.get(2)?,
            description: row.get(3)?,
            location: row.get(4)?,
            start,
            end,
            all_day: all_day != 0,
            attendees: serde_json::from_str(&attendees_json).unwrap_or_default(),
            organizer: row.get(9)?,
            status,
            html_link: row.get(11)?,
            etag: row.get(12)?,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn create_test_event(id: &str, summary: &str, start_hours_from_now: i64) -> Event {
        let start = Utc::now() + chrono::Duration::hours(start_hours_from_now);
        let end = start + chrono::Duration::hours(1);

        Event {
            id: id.to_string(),
            calendar_id: "primary".to_string(),
            summary: summary.to_string(),
            description: Some("Test event".to_string()),
            location: Some("Room A".to_string()),
            start: EventTime::DateTime(start),
            end: EventTime::DateTime(end),
            all_day: false,
            attendees: vec![],
            organizer: Some("organizer@example.com".to_string()),
            status: EventStatus::Confirmed,
            html_link: None,
            etag: None,
        }
    }

    #[test]
    fn test_store_and_get_event() {
        let cache = CalendarCache::in_memory().unwrap();
        let event = create_test_event("event1", "Meeting", 1);

        cache.store_event(&event).unwrap();
        let retrieved = cache.get_event("primary", "event1").unwrap().unwrap();

        assert_eq!(retrieved.id, "event1");
        assert_eq!(retrieved.summary, "Meeting");
    }

    #[test]
    fn test_event_not_found() {
        let cache = CalendarCache::in_memory().unwrap();
        let result = cache.get_event("primary", "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_events_in_range() {
        let cache = CalendarCache::in_memory().unwrap();

        cache.store_event(&create_test_event("e1", "Event 1", 1)).unwrap();
        cache.store_event(&create_test_event("e2", "Event 2", 2)).unwrap();
        cache.store_event(&create_test_event("e3", "Event 3", 48)).unwrap(); // 2 days from now

        let now = Utc::now();
        let tomorrow = now + chrono::Duration::hours(24);
        let events = cache.list_events("primary", now, tomorrow).unwrap();

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_delete_event() {
        let cache = CalendarCache::in_memory().unwrap();
        let event = create_test_event("event1", "To Delete", 1);

        cache.store_event(&event).unwrap();
        assert!(cache.get_event("primary", "event1").unwrap().is_some());

        cache.delete_event("primary", "event1").unwrap();
        assert!(cache.get_event("primary", "event1").unwrap().is_none());
    }

    #[test]
    fn test_upcoming_event_count() {
        let cache = CalendarCache::in_memory().unwrap();

        cache.store_event(&create_test_event("e1", "Event 1", 1)).unwrap();
        cache.store_event(&create_test_event("e2", "Event 2", 2)).unwrap();
        cache.store_event(&create_test_event("e3", "Event 3", 48)).unwrap();

        let count = cache.upcoming_event_count("primary", 24).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_store_and_list_calendars() {
        let cache = CalendarCache::in_memory().unwrap();

        let cal1 = Calendar {
            id: "primary".to_string(),
            summary: "My Calendar".to_string(),
            description: None,
            time_zone: Some("America/New_York".to_string()),
            background_color: None,
            foreground_color: None,
            is_primary: true,
            access_role: AccessRole::Owner,
        };
        let cal2 = Calendar {
            id: "work".to_string(),
            summary: "Work".to_string(),
            description: None,
            time_zone: None,
            background_color: None,
            foreground_color: None,
            is_primary: false,
            access_role: AccessRole::Writer,
        };

        cache.store_calendar(&cal1).unwrap();
        cache.store_calendar(&cal2).unwrap();

        let calendars = cache.list_calendars().unwrap();
        assert_eq!(calendars.len(), 2);
    }

    #[test]
    fn test_clear() {
        let cache = CalendarCache::in_memory().unwrap();

        cache.store_event(&create_test_event("e1", "Event", 1)).unwrap();
        cache.clear().unwrap();

        assert!(cache.get_event("primary", "e1").unwrap().is_none());
    }
}
