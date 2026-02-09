//! Weather service for MyMe
//!
//! Provides weather data via Open-Meteo API with system location detection
//! and persistent caching.

pub mod types;
pub mod cache;
pub mod geocode;
pub mod location;
pub mod provider;

pub use types::*;
pub use cache::WeatherCache;
pub use geocode::reverse_geocode;
pub use provider::WeatherProvider;
