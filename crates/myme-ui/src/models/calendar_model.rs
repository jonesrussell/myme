//! Calendar model for QML.
//!
//! Provides event listing and management.

use core::pin::Pin;
use std::sync::mpsc;

use chrono::{Duration, Utc};
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::{SecureStorage, GoogleOAuth2Provider};
use myme_calendar::{CalendarClient, CalendarCache, Event, Calendar};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(bool, authenticated)]
        #[qproperty(QString, error_message)]
        #[qproperty(i32, event_count)]
        #[qproperty(i32, today_event_count)]
        #[qproperty(QString, next_event_summary)]
        #[qproperty(QString, next_event_time)]
        type CalendarModel = super::CalendarModelRust;

        #[qinvokable]
        fn check_auth(self: Pin<&mut CalendarModel>);

        #[qinvokable]
        fn fetch_events(self: Pin<&mut CalendarModel>);

        #[qinvokable]
        fn fetch_today_events(self: Pin<&mut CalendarModel>);

        #[qinvokable]
        fn get_event(self: Pin<&mut CalendarModel>, index: i32) -> QString;

        #[qinvokable]
        fn get_calendars(self: Pin<&mut CalendarModel>) -> QString;

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut CalendarModel>);

        #[qsignal]
        fn events_changed(self: Pin<&mut CalendarModel>);

        #[qsignal]
        fn calendars_changed(self: Pin<&mut CalendarModel>);
    }
}

/// Messages for async operations
enum CalendarMessage {
    FetchEventsDone(Result<Vec<Event>, String>),
    FetchCalendarsDone(Result<Vec<Calendar>, String>),
}

#[derive(Default)]
pub struct CalendarModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    event_count: i32,
    today_event_count: i32,
    next_event_summary: QString,
    next_event_time: QString,
    events: Vec<Event>,
    calendars: Vec<Calendar>,
    rx: Option<mpsc::Receiver<CalendarMessage>>,
}

impl CalendarModelRust {
    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_access_token() -> Option<String> {
        let token_set = SecureStorage::retrieve_token("google").ok()?;

        if token_set.is_expired() {
            // Try to refresh
            if let Some(refresh_token) = &token_set.refresh_token {
                if let Some((client_id, client_secret)) = get_google_config() {
                    let rt = tokio::runtime::Runtime::new().ok()?;
                    let provider = GoogleOAuth2Provider::new(client_id, client_secret);

                    if let Ok(new_tokens) = rt.block_on(provider.refresh_token(refresh_token)) {
                        let expires_at = chrono::Utc::now().timestamp() + new_tokens.expires_in as i64;
                        let new_token_set = myme_auth::TokenSet {
                            access_token: new_tokens.access_token.clone(),
                            refresh_token: new_tokens.refresh_token.or(token_set.refresh_token.clone()),
                            expires_at,
                            scopes: new_tokens.scope.split(' ').map(|s| s.to_string()).collect(),
                        };
                        let _ = SecureStorage::store_token("google", &new_token_set);
                        return Some(new_tokens.access_token);
                    }
                }
            }
            return None;
        }

        Some(token_set.access_token)
    }

    fn get_cache_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("myme")
            .join("calendar_cache.db")
    }
}

fn get_google_config() -> Option<(String, String)> {
    match myme_core::Config::load() {
        Ok(config) => {
            let client_id = config.google.as_ref()?.client_id.clone()?;
            let client_secret = config.google.as_ref()?.client_secret.clone()?;
            Some((client_id, client_secret))
        }
        Err(_) => None,
    }
}

impl qobject::CalendarModel {
    /// Check if Google is authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let is_authenticated = SecureStorage::has_token("google");
        self.as_mut().set_authenticated(is_authenticated);

        if is_authenticated {
            // Load cached event count
            if let Ok(cache) = CalendarCache::new(CalendarModelRust::get_cache_path()) {
                if let Ok(count) = cache.upcoming_event_count("primary", 24) {
                    self.as_mut().set_today_event_count(count as i32);
                }
            }
        }
    }

    /// Fetch events for the next 7 days (non-blocking)
    pub fn fetch_events(mut self: Pin<&mut Self>) {
        let access_token = match CalendarModelRust::get_access_token() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                self.as_mut().set_authenticated(false);
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        let cache_path = CalendarModelRust::get_cache_path();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = CalendarClient::new(&access_token);

                let time_min = Utc::now();
                let time_max = time_min + Duration::days(7);

                // Fetch events from primary calendar
                let response = client.list_events("primary", time_min, time_max, None).await
                    .map_err(|e| e.to_string())?;

                let events: Vec<Event> = response.items
                    .into_iter()
                    .map(|api_event| Event::from_api(api_event, "primary"))
                    .collect();

                // Cache events
                if let Ok(cache) = CalendarCache::new(&cache_path) {
                    for event in &events {
                        let _ = cache.store_event(event);
                    }
                }

                Ok(events)
            });

            let _ = tx.send(CalendarMessage::FetchEventsDone(result));
        });
    }

    /// Fetch events for today only
    pub fn fetch_today_events(mut self: Pin<&mut Self>) {
        let access_token = match CalendarModelRust::get_access_token() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                self.as_mut().set_authenticated(false);
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = CalendarClient::new(&access_token);

                let today = Utc::now().date_naive();
                let time_min = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let time_max = today.and_hms_opt(23, 59, 59).unwrap().and_utc();

                let response = client.list_events("primary", time_min, time_max, None).await
                    .map_err(|e| e.to_string())?;

                let events: Vec<Event> = response.items
                    .into_iter()
                    .map(|api_event| Event::from_api(api_event, "primary"))
                    .collect();

                Ok(events)
            });

            let _ = tx.send(CalendarMessage::FetchEventsDone(result));
        });
    }

    /// Get event at index as JSON
    pub fn get_event(self: Pin<&mut Self>, index: i32) -> QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.events.len() {
            return QString::from("{}");
        }

        let event = &rust.events[index as usize];
        let json = serde_json::json!({
            "id": event.id,
            "summary": event.summary,
            "description": event.description,
            "location": event.location,
            "start": event.start.as_datetime().to_rfc3339(),
            "end": event.end.as_datetime().to_rfc3339(),
            "allDay": event.all_day,
            "status": format!("{:?}", event.status),
        });

        QString::from(json.to_string().as_str())
    }

    /// Get calendars as JSON
    pub fn get_calendars(self: Pin<&mut Self>) -> QString {
        let calendars: Vec<_> = self.as_ref().rust().calendars.iter().map(|cal| {
            serde_json::json!({
                "id": cal.id,
                "summary": cal.summary,
                "isPrimary": cal.is_primary,
                "backgroundColor": cal.background_color,
            })
        }).collect();

        QString::from(serde_json::to_string(&calendars).unwrap_or_else(|_| "[]".to_string()).as_str())
    }

    /// Poll for async operation results
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match self.as_ref().rust().rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            Some(m) => m,
            None => return,
        };

        match msg {
            CalendarMessage::FetchEventsDone(result) => {
                self.as_mut().set_loading(false);

                match result {
                    Ok(events) => {
                        let now = Utc::now();
                        let today = now.date_naive();
                        let today_count = events.iter().filter(|e| {
                            e.start.as_datetime().date_naive() == today
                        }).count();

                        // Find next upcoming event
                        let next_event = events.iter()
                            .filter(|e| e.start.as_datetime() > now)
                            .min_by_key(|e| e.start.as_datetime());

                        if let Some(event) = next_event {
                            let summary = if event.summary.is_empty() { "(No title)" } else { &event.summary };
                            self.as_mut().set_next_event_summary(QString::from(summary));
                            let time_str = event.start.as_datetime()
                                .format("%H:%M")
                                .to_string();
                            self.as_mut().set_next_event_time(QString::from(time_str.as_str()));
                        } else {
                            self.as_mut().set_next_event_summary(QString::from(""));
                            self.as_mut().set_next_event_time(QString::from(""));
                        }

                        self.as_mut().set_event_count(events.len() as i32);
                        self.as_mut().set_today_event_count(today_count as i32);
                        self.as_mut().rust_mut().events = events;
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().events_changed();
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(&e);
                    }
                }
            }
            CalendarMessage::FetchCalendarsDone(result) => {
                self.as_mut().set_loading(false);

                match result {
                    Ok(calendars) => {
                        self.as_mut().rust_mut().calendars = calendars;
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().calendars_changed();
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(&e);
                    }
                }
            }
        }
    }
}
