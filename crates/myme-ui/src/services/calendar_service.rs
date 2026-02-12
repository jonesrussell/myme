//! Calendar backend: async operations using the shared runtime and channel pattern.
//! All network work runs off the UI thread; results sent via mpsc.

use std::path::PathBuf;

use chrono::{Duration, Utc};
use myme_calendar::{Calendar, CalendarCache, CalendarClient, Event};

use crate::bridge;

/// Error type for Calendar operations.
#[derive(Debug, Clone)]
pub enum CalendarError {
    Network(String),
    Auth(String),
    NotInitialized,
}

impl std::fmt::Display for CalendarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalendarError::Network(s) => write!(f, "Calendar error: {}", s),
            CalendarError::Auth(s) => write!(f, "Calendar auth error: {}", s),
            CalendarError::NotInitialized => write!(f, "Calendar service not initialized"),
        }
    }
}

impl std::error::Error for CalendarError {}

/// Messages sent from async operations back to the UI thread.
#[derive(Debug)]
pub enum CalendarServiceMessage {
    /// Result of fetching events.
    FetchEventsDone(Result<Vec<Event>, CalendarError>),
    /// Result of fetching calendar list.
    FetchCalendarsDone(Result<Vec<Calendar>, CalendarError>),
}

/// Request to fetch events for the next 7 days.
pub fn request_fetch_events(
    tx: &std::sync::mpsc::Sender<CalendarServiceMessage>,
    access_token: String,
    cache_path: PathBuf,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(CalendarServiceMessage::FetchEventsDone(Err(
                CalendarError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = CalendarClient::new(&access_token);
        let time_min = Utc::now();
        let time_max = time_min + Duration::days(7);

        let result = client
            .list_events("primary", time_min, time_max, None)
            .await
            .map_err(|e| CalendarError::Network(e.to_string()))
            .map(|response| {
                response
                    .items
                    .into_iter()
                    .map(|api_event| Event::from_api(api_event, "primary"))
                    .collect::<Vec<Event>>()
            });

        if let Ok(ref events) = result {
            if let Ok(cache) = CalendarCache::new(&cache_path) {
                for event in events {
                    let _ = cache.store_event(event);
                }
            }
        }

        let _ = tx.send(CalendarServiceMessage::FetchEventsDone(result));
    });
}

/// Request to fetch events for today only.
pub fn request_fetch_today_events(
    tx: &std::sync::mpsc::Sender<CalendarServiceMessage>,
    access_token: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(CalendarServiceMessage::FetchEventsDone(Err(
                CalendarError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = CalendarClient::new(&access_token);
        let today = Utc::now().date_naive();
        let time_min = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let time_max = today.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let result = client
            .list_events("primary", time_min, time_max, None)
            .await
            .map_err(|e| CalendarError::Network(e.to_string()))
            .map(|response| {
                response
                    .items
                    .into_iter()
                    .map(|api_event| Event::from_api(api_event, "primary"))
                    .collect::<Vec<Event>>()
            });

        let _ = tx.send(CalendarServiceMessage::FetchEventsDone(result));
    });
}
