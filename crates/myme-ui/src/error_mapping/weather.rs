use crate::services::weather_service::WeatherError as UiWeatherError;
use myme_core::{AppError, WeatherError};

impl From<UiWeatherError> for AppError {
    fn from(e: UiWeatherError) -> Self {
        match e {
            UiWeatherError::Network(s) => AppError::Weather(WeatherError::ApiError(s)),
            UiWeatherError::Location(s) => AppError::Weather(WeatherError::LocationNotFound(s)),
            UiWeatherError::NotInitialized => AppError::Weather(WeatherError::ServiceUnavailable),
        }
    }
}
