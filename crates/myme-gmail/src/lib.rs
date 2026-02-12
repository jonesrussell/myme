//! Gmail integration for MyMe.
//!
//! Provides Gmail API client, offline caching, and sync queue.

pub mod cache;
pub mod client;
pub mod error;
pub mod sync;
pub mod types;

pub use cache::GmailCache;
pub use client::GmailClient;
pub use error::GmailError;
pub use sync::{QueuedAction, SyncAction, SyncQueue};
pub use types::{Label, LabelType, Message, MessageListResponse, MessageRef};
