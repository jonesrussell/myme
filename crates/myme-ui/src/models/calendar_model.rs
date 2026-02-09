//! Calendar model for QML.
//!
//! Provides event listing and management.
//! Uses the shared AppServices runtime and channel pattern (no block_on).

use core::pin::Pin;

use chrono::Utc;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::SecureStorage;
use myme_calendar::{Calendar, CalendarCache, Event};

use crate::bridge;
use crate::services::google_common::{get_google_access_token, get_google_cache_path};
use crate::services::{
    request_calendar_fetch_events, request_calendar_fetch_today_events,
    CalendarServiceMessage,
};

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
}

impl CalendarModelRust {
    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_access_token() -> Option<String> {
        get_google_access_token()
    }

    fn get_cache_path() -> std::path::PathBuf {
        get_google_cache_path("calendar_cache.db")
    }
}

impl qobject::CalendarModel {
    /// Check if Google is authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let is_authenticated = SecureStorage::has_token("google");
        self.as_mut().set_authenticated(is_authenticated);

        if is_authenticated {
            if let Ok(cache) = CalendarCache::new(CalendarModelRust::get_cache_path()) {
                if let Ok(count) = cache.upcoming_event_count("primary", 24) {
                    self.as_mut().set_today_event_count(count as i32);
                }
            }
        }
    }

    /// Fetch events for the next 7 days (non-blocking, uses shared runtime)
    pub fn fetch_events(mut self: Pin<&mut Self>) {
        let access_token = match CalendarModelRust::get_access_token() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                self.as_mut().set_authenticated(false);
                return;
            }
        };

        bridge::init_calendar_service_channel();
        let tx = match bridge::get_calendar_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        let cache_path = CalendarModelRust::get_cache_path();
        request_calendar_fetch_events(&tx, access_token, cache_path);
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

        bridge::init_calendar_service_channel();
        let tx = match bridge::get_calendar_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        request_calendar_fetch_today_events(&tx, access_token);
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
        let calendars: Vec<_> = self
            .as_ref()
            .rust()
            .calendars
            .iter()
            .map(|cal| {
                serde_json::json!({
                    "id": cal.id,
                    "summary": cal.summary,
                    "isPrimary": cal.is_primary,
                    "backgroundColor": cal.background_color,
                })
            })
            .collect();

        QString::from(
            serde_json::to_string(&calendars).unwrap_or_else(|_| "[]".to_string()).as_str(),
        )
    }

    /// Poll for async operation results
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_calendar_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            CalendarServiceMessage::FetchEventsDone(result) => {
                self.as_mut().set_loading(false);

                match result {
                    Ok(events) => {
                        let now = Utc::now();
                        let today = now.date_naive();
                        let today_count = events
                            .iter()
                            .filter(|e| e.start.as_datetime().date_naive() == today)
                            .count();

                        let next_event = events
                            .iter()
                            .filter(|e| e.start.as_datetime() > now)
                            .min_by_key(|e| e.start.as_datetime());

                        if let Some(event) = next_event {
                            let summary = if event.summary.is_empty() {
                                "(No title)"
                            } else {
                                &event.summary
                            };
                            self.as_mut()
                                .set_next_event_summary(QString::from(summary));
                            let time_str = event.start.as_datetime().format("%H:%M").to_string();
                            self.as_mut()
                                .set_next_event_time(QString::from(time_str.as_str()));
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
            CalendarServiceMessage::FetchCalendarsDone(result) => {
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
