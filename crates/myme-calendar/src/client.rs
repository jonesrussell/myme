//! Google Calendar API client.

use chrono::{DateTime, Utc};
use tracing::instrument;

use crate::error::CalendarError;
use crate::types::*;

const CALENDAR_API_BASE: &str = "https://www.googleapis.com/calendar/v3";

pub struct CalendarClient {
    client: reqwest::Client,
    access_token: String,
    base_url: String,
}

impl CalendarClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: access_token.to_string(),
            base_url: CALENDAR_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(access_token: &str, base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: access_token.to_string(),
            base_url: base_url.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    /// List all calendars.
    #[instrument(skip(self), level = "info")]
    pub async fn list_calendars(&self) -> Result<Vec<Calendar>, CalendarError> {
        let url = format!("{}/users/me/calendarList", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let resp: CalendarListResponse = self.handle_response(response).await?;
        Ok(resp.items.into_iter().map(Calendar::from).collect())
    }

    /// List events from a calendar within a time range.
    #[instrument(skip(self), level = "info")]
    pub async fn list_events(
        &self,
        calendar_id: &str,
        time_min: DateTime<Utc>,
        time_max: DateTime<Utc>,
        page_token: Option<&str>,
    ) -> Result<EventListResponse, CalendarError> {
        let mut url = format!(
            "{}/calendars/{}/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime&maxResults=50",
            self.base_url,
            urlencoding::encode(calendar_id),
            urlencoding::encode(&time_min.to_rfc3339()),
            urlencoding::encode(&time_max.to_rfc3339()),
        );

        if let Some(pt) = page_token {
            url.push_str(&format!("&pageToken={}", pt));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a single event.
    #[instrument(skip(self), level = "info")]
    pub async fn get_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<Event, CalendarError> {
        let url = format!(
            "{}/calendars/{}/events/{}",
            self.base_url,
            urlencoding::encode(calendar_id),
            urlencoding::encode(event_id),
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let api_event: ApiEvent = self.handle_response(response).await?;
        Ok(Event::from_api(api_event, calendar_id))
    }

    /// Create a new event.
    #[instrument(skip(self), level = "info")]
    pub async fn create_event(
        &self,
        calendar_id: &str,
        summary: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        description: Option<&str>,
        location: Option<&str>,
    ) -> Result<Event, CalendarError> {
        let url = format!(
            "{}/calendars/{}/events",
            self.base_url,
            urlencoding::encode(calendar_id),
        );

        let mut body = serde_json::json!({
            "summary": summary,
            "start": { "dateTime": start.to_rfc3339() },
            "end": { "dateTime": end.to_rfc3339() },
        });

        if let Some(desc) = description {
            body["description"] = serde_json::Value::String(desc.to_string());
        }
        if let Some(loc) = location {
            body["location"] = serde_json::Value::String(loc.to_string());
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        let api_event: ApiEvent = self.handle_response(response).await?;
        Ok(Event::from_api(api_event, calendar_id))
    }

    /// Update an existing event.
    #[instrument(skip(self), level = "info")]
    pub async fn update_event(
        &self,
        calendar_id: &str,
        event_id: &str,
        summary: Option<&str>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        description: Option<&str>,
        location: Option<&str>,
    ) -> Result<Event, CalendarError> {
        let url = format!(
            "{}/calendars/{}/events/{}",
            self.base_url,
            urlencoding::encode(calendar_id),
            urlencoding::encode(event_id),
        );

        let mut body = serde_json::Map::new();

        if let Some(s) = summary {
            body.insert(
                "summary".to_string(),
                serde_json::Value::String(s.to_string()),
            );
        }
        if let Some(s) = start {
            body.insert(
                "start".to_string(),
                serde_json::json!({ "dateTime": s.to_rfc3339() }),
            );
        }
        if let Some(e) = end {
            body.insert(
                "end".to_string(),
                serde_json::json!({ "dateTime": e.to_rfc3339() }),
            );
        }
        if let Some(d) = description {
            body.insert(
                "description".to_string(),
                serde_json::Value::String(d.to_string()),
            );
        }
        if let Some(l) = location {
            body.insert(
                "location".to_string(),
                serde_json::Value::String(l.to_string()),
            );
        }

        let response = self
            .client
            .patch(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        let api_event: ApiEvent = self.handle_response(response).await?;
        Ok(Event::from_api(api_event, calendar_id))
    }

    /// Delete an event.
    #[instrument(skip(self), level = "info")]
    pub async fn delete_event(
        &self,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<(), CalendarError> {
        let url = format!(
            "{}/calendars/{}/events/{}",
            self.base_url,
            urlencoding::encode(calendar_id),
            urlencoding::encode(event_id),
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        // Delete returns 204 No Content on success
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(CalendarError::ApiError(format!("{}: {}", status, text)))
        }
    }

    /// Quick add an event using natural language.
    #[instrument(skip(self), level = "info")]
    pub async fn quick_add(&self, calendar_id: &str, text: &str) -> Result<Event, CalendarError> {
        let url = format!(
            "{}/calendars/{}/events/quickAdd?text={}",
            self.base_url,
            urlencoding::encode(calendar_id),
            urlencoding::encode(text),
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let api_event: ApiEvent = self.handle_response(response).await?;
        Ok(Event::from_api(api_event, calendar_id))
    }

    /// Helper to handle API responses and errors.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, CalendarError> {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| CalendarError::ApiError(format!("JSON parse error: {}", e)))
        } else if status.as_u16() == 401 {
            Err(CalendarError::TokenExpired)
        } else if status.as_u16() == 403 {
            Err(CalendarError::AuthRequired)
        } else if status.as_u16() == 404 {
            let text = response.text().await.unwrap_or_default();
            Err(CalendarError::EventNotFound(text))
        } else if status.as_u16() == 409 {
            Err(CalendarError::Conflict)
        } else if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            Err(CalendarError::RateLimited(retry_after))
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(CalendarError::ApiError(format!("{}: {}", status, text)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_calendars() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/users/me/calendarList"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {"id": "primary", "summary": "My Calendar", "primary": true, "accessRole": "owner"},
                    {"id": "cal2", "summary": "Work", "accessRole": "writer"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("test_token", &mock_server.uri());
        let calendars = client.list_calendars().await.unwrap();

        assert_eq!(calendars.len(), 2);
        assert!(calendars[0].is_primary);
    }

    #[tokio::test]
    async fn test_list_events() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/calendars/primary/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [
                    {
                        "id": "event1",
                        "summary": "Meeting",
                        "start": {"dateTime": "2024-02-01T10:00:00Z"},
                        "end": {"dateTime": "2024-02-01T11:00:00Z"}
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("test_token", &mock_server.uri());
        let time_min = DateTime::parse_from_rfc3339("2024-02-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let time_max = DateTime::parse_from_rfc3339("2024-02-28T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);

        let response = client
            .list_events("primary", time_min, time_max, None)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].summary, Some("Meeting".to_string()));
    }

    #[tokio::test]
    async fn test_get_event() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/calendars/primary/events/event123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "event123",
                "summary": "Team Sync",
                "start": {"dateTime": "2024-02-01T14:00:00Z"},
                "end": {"dateTime": "2024-02-01T15:00:00Z"},
                "status": "confirmed"
            })))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("test_token", &mock_server.uri());
        let event = client.get_event("primary", "event123").await.unwrap();

        assert_eq!(event.id, "event123");
        assert_eq!(event.summary, "Team Sync");
    }

    #[tokio::test]
    async fn test_token_expired() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/users/me/calendarList"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("expired_token", &mock_server.uri());
        let result = client.list_calendars().await;

        assert!(matches!(result, Err(CalendarError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/users/me/calendarList"))
            .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "60"))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("token", &mock_server.uri());
        let result = client.list_calendars().await;

        assert!(matches!(result, Err(CalendarError::RateLimited(60))));
    }

    #[tokio::test]
    async fn test_delete_event() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/calendars/primary/events/event123"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = CalendarClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.delete_event("primary", "event123").await;

        assert!(result.is_ok());
    }
}
