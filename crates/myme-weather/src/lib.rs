//! Weather service for MyMe
//!
//! Provides weather data via Open-Meteo API with system location detection
//! and persistent caching.

pub mod cache;
pub mod location;
pub mod provider;
pub mod types;

pub use cache::WeatherCache;
pub use provider::WeatherProvider;
pub use types::*;
