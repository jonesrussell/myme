//! Google Calendar integration for MyMe.
//!
//! Provides Calendar API client and offline caching.

pub mod cache;
pub mod client;
pub mod error;
pub mod types;

pub use cache::CalendarCache;
pub use client::CalendarClient;
pub use error::CalendarError;
pub use types::{AccessRole, Attendee, Calendar, Event, EventStatus, EventTime, ResponseStatus};
