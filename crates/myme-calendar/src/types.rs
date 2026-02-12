//! Calendar API types and data structures.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Calendar event as stored locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub calendar_id: String,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
    pub all_day: bool,
    pub attendees: Vec<Attendee>,
    pub organizer: Option<String>,
    pub status: EventStatus,
    pub html_link: Option<String>,
    pub etag: Option<String>,
}

/// Event time - can be a specific datetime or an all-day date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTime {
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
}

impl EventTime {
    pub fn as_datetime(&self) -> DateTime<Utc> {
        match self {
            EventTime::DateTime(dt) => *dt,
            EventTime::Date(d) => d.and_hms_opt(0, 0, 0).unwrap().and_utc(),
        }
    }
}

/// Event status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
}

impl Default for EventStatus {
    fn default() -> Self {
        Self::Confirmed
    }
}

/// Event attendee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendee {
    pub email: String,
    pub display_name: Option<String>,
    pub response_status: ResponseStatus,
    pub is_organizer: bool,
}

/// Attendee response status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    NeedsAction,
    Declined,
    Tentative,
    Accepted,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        Self::NeedsAction
    }
}

/// Calendar metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub time_zone: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub is_primary: bool,
    pub access_role: AccessRole,
}

/// Calendar access role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessRole {
    Owner,
    Writer,
    Reader,
    FreeBusyReader,
}

impl Default for AccessRole {
    fn default() -> Self {
        Self::Reader
    }
}

// API Response Types

/// Google Calendar API event response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEvent {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: Option<ApiEventTime>,
    pub end: Option<ApiEventTime>,
    #[serde(default)]
    pub attendees: Vec<ApiAttendee>,
    pub organizer: Option<ApiOrganizer>,
    pub status: Option<String>,
    pub html_link: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiEventTime {
    pub date_time: Option<String>,
    pub date: Option<String>,
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAttendee {
    pub email: String,
    pub display_name: Option<String>,
    pub response_status: Option<String>,
    #[serde(default)]
    pub organizer: bool,
}

#[derive(Debug, Deserialize)]
pub struct ApiOrganizer {
    pub email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

/// API response for event list.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventListResponse {
    #[serde(default)]
    pub items: Vec<ApiEvent>,
    pub next_page_token: Option<String>,
    pub next_sync_token: Option<String>,
}

/// API response for calendar list.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarListResponse {
    #[serde(default)]
    pub items: Vec<ApiCalendar>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiCalendar {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub time_zone: Option<String>,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    #[serde(default)]
    pub primary: bool,
    pub access_role: Option<String>,
}

impl Event {
    /// Convert API response to local Event.
    pub fn from_api(api: ApiEvent, calendar_id: &str) -> Self {
        let (start, all_day) = api
            .start
            .map(|t| parse_event_time(&t))
            .unwrap_or((EventTime::DateTime(Utc::now()), false));

        let end = api
            .end
            .map(|t| parse_event_time(&t).0)
            .unwrap_or_else(|| start.clone());

        let status = match api.status.as_deref() {
            Some("confirmed") => EventStatus::Confirmed,
            Some("tentative") => EventStatus::Tentative,
            Some("cancelled") => EventStatus::Cancelled,
            _ => EventStatus::Confirmed,
        };

        let attendees = api
            .attendees
            .into_iter()
            .map(|a| {
                let response_status = match a.response_status.as_deref() {
                    Some("accepted") => ResponseStatus::Accepted,
                    Some("declined") => ResponseStatus::Declined,
                    Some("tentative") => ResponseStatus::Tentative,
                    _ => ResponseStatus::NeedsAction,
                };
                Attendee {
                    email: a.email,
                    display_name: a.display_name,
                    response_status,
                    is_organizer: a.organizer,
                }
            })
            .collect();

        Self {
            id: api.id,
            calendar_id: calendar_id.to_string(),
            summary: api.summary.unwrap_or_default(),
            description: api.description,
            location: api.location,
            start,
            end,
            all_day,
            attendees,
            organizer: api.organizer.and_then(|o| o.email),
            status,
            html_link: api.html_link,
            etag: api.etag,
        }
    }
}

impl From<ApiCalendar> for Calendar {
    fn from(api: ApiCalendar) -> Self {
        let access_role = match api.access_role.as_deref() {
            Some("owner") => AccessRole::Owner,
            Some("writer") => AccessRole::Writer,
            Some("reader") => AccessRole::Reader,
            Some("freeBusyReader") => AccessRole::FreeBusyReader,
            _ => AccessRole::Reader,
        };

        Self {
            id: api.id,
            summary: api.summary.unwrap_or_default(),
            description: api.description,
            time_zone: api.time_zone,
            background_color: api.background_color,
            foreground_color: api.foreground_color,
            is_primary: api.primary,
            access_role,
        }
    }
}

fn parse_event_time(api: &ApiEventTime) -> (EventTime, bool) {
    if let Some(dt_str) = &api.date_time {
        // Try parsing as RFC3339
        if let Ok(dt) = DateTime::parse_from_rfc3339(dt_str) {
            return (EventTime::DateTime(dt.with_timezone(&Utc)), false);
        }
    }
    if let Some(date_str) = &api.date {
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            return (EventTime::Date(date), true);
        }
    }
    (EventTime::DateTime(Utc::now()), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_from_api() {
        let json = r#"{
            "id": "event123",
            "summary": "Team Meeting",
            "description": "Weekly sync",
            "location": "Conference Room A",
            "start": {"dateTime": "2024-02-01T10:00:00Z"},
            "end": {"dateTime": "2024-02-01T11:00:00Z"},
            "status": "confirmed",
            "htmlLink": "https://calendar.google.com/event?id=123"
        }"#;

        let api_event: ApiEvent = serde_json::from_str(json).unwrap();
        let event = Event::from_api(api_event, "primary");

        assert_eq!(event.id, "event123");
        assert_eq!(event.summary, "Team Meeting");
        assert_eq!(event.location, Some("Conference Room A".to_string()));
        assert_eq!(event.status, EventStatus::Confirmed);
        assert!(!event.all_day);
    }

    #[test]
    fn test_all_day_event() {
        let json = r#"{
            "id": "event456",
            "summary": "Holiday",
            "start": {"date": "2024-02-01"},
            "end": {"date": "2024-02-02"}
        }"#;

        let api_event: ApiEvent = serde_json::from_str(json).unwrap();
        let event = Event::from_api(api_event, "primary");

        assert!(event.all_day);
        assert!(matches!(event.start, EventTime::Date(_)));
    }

    #[test]
    fn test_calendar_from_api() {
        let json = r#"{
            "id": "primary",
            "summary": "My Calendar",
            "timeZone": "America/New_York",
            "primary": true,
            "accessRole": "owner"
        }"#;

        let api_calendar: ApiCalendar = serde_json::from_str(json).unwrap();
        let calendar = Calendar::from(api_calendar);

        assert_eq!(calendar.id, "primary");
        assert!(calendar.is_primary);
        assert_eq!(calendar.access_role, AccessRole::Owner);
    }

    #[test]
    fn test_event_with_attendees() {
        let json = r#"{
            "id": "event789",
            "summary": "Project Review",
            "start": {"dateTime": "2024-02-01T14:00:00Z"},
            "end": {"dateTime": "2024-02-01T15:00:00Z"},
            "attendees": [
                {"email": "alice@example.com", "responseStatus": "accepted", "organizer": true},
                {"email": "bob@example.com", "responseStatus": "tentative"}
            ],
            "organizer": {"email": "alice@example.com"}
        }"#;

        let api_event: ApiEvent = serde_json::from_str(json).unwrap();
        let event = Event::from_api(api_event, "primary");

        assert_eq!(event.attendees.len(), 2);
        assert_eq!(event.attendees[0].response_status, ResponseStatus::Accepted);
        assert!(event.attendees[0].is_organizer);
        assert_eq!(event.organizer, Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_event_time_as_datetime() {
        let dt = EventTime::DateTime(Utc::now());
        assert!(dt.as_datetime() <= Utc::now());

        let date = EventTime::Date(NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        let as_dt = date.as_datetime();
        assert_eq!(
            as_dt.date_naive(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
        );
    }
}
