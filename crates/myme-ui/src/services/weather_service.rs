//! Weather backend: async weather fetching.
//! All network work runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_weather::{WeatherData, WeatherProvider};

use crate::bridge;

/// Error type for weather operations
#[derive(Debug, Clone)]
pub enum WeatherError {
    Network(String),
    Location(String),
    NotInitialized,
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::Network(s) => write!(f, "Weather error: {}", s),
            WeatherError::Location(s) => write!(f, "Location error: {}", s),
            WeatherError::NotInitialized => write!(f, "Weather service not initialized"),
        }
    }
}

impl std::error::Error for WeatherError {}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum WeatherServiceMessage {
    /// Result of fetching weather data
    FetchDone(Result<WeatherData, WeatherError>),
}

/// Request to fetch weather data asynchronously.
/// Sends `FetchDone` on the channel when complete.
pub fn request_fetch(
    tx: &std::sync::mpsc::Sender<WeatherServiceMessage>,
    provider: Arc<WeatherProvider>,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(WeatherServiceMessage::FetchDone(Err(
                WeatherError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        // First get location
        let location = match myme_weather::location::get_current_location().await {
            Ok(loc) => {
                tracing::info!("Got location: {}, {}", loc.latitude, loc.longitude);
                loc
            }
            Err(e) => {
                let _ = tx.send(WeatherServiceMessage::FetchDone(Err(WeatherError::Location(
                    e.to_string(),
                ))));
                return;
            }
        };

        // Then fetch weather
        let result = provider
            .fetch(&location)
            .await
            .map_err(|e| WeatherError::Network(e.to_string()));
        let _ = tx.send(WeatherServiceMessage::FetchDone(result));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_error_display() {
        assert!(format!("{}", WeatherError::Network("timeout".into())).contains("Weather"));
        assert!(format!("{}", WeatherError::Location("failed".into())).contains("Location"));
        assert!(format!("{}", WeatherError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn weather_service_message_variants() {
        let _fetch_err: WeatherServiceMessage =
            WeatherServiceMessage::FetchDone(Err(WeatherError::NotInitialized));
    }
}
