// Provider implementation - placeholder for Task 4
use crate::types::{Location, TemperatureUnit, WeatherData, WeatherError};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WeatherProvider {
    client: Arc<Client>,
    unit: TemperatureUnit,
}

impl WeatherProvider {
    pub fn new(unit: TemperatureUnit) -> Result<Self, WeatherError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client: Arc::new(client),
            unit,
        })
    }
}
