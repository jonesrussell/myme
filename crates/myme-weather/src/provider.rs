use crate::types::*;
use chrono::{NaiveDate, NaiveTime, Utc};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

const OPEN_METEO_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";
const REQUEST_TIMEOUT_SECS: u64 = 10;

/// Open-Meteo API response structures
mod api {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct ForecastResponse {
        pub current: CurrentData,
        pub daily: DailyData,
        pub hourly: HourlyData,
    }

    #[derive(Debug, Deserialize)]
    pub struct CurrentData {
        pub temperature_2m: f64,
        pub apparent_temperature: f64,
        pub relative_humidity_2m: i32,
        pub wind_speed_10m: f64,
        pub weather_code: i32,
    }

    #[derive(Debug, Deserialize)]
    pub struct DailyData {
        pub time: Vec<String>,
        pub temperature_2m_max: Vec<f64>,
        pub temperature_2m_min: Vec<f64>,
        pub weather_code: Vec<i32>,
        pub precipitation_probability_max: Vec<i32>,
        pub sunrise: Vec<String>,
        pub sunset: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct HourlyData {
        pub time: Vec<String>,
        pub temperature_2m: Vec<f64>,
        pub weather_code: Vec<i32>,
        pub precipitation_probability: Vec<i32>,
    }
}

/// Weather data provider using Open-Meteo API
#[derive(Debug, Clone)]
pub struct WeatherProvider {
    client: Arc<Client>,
    unit: TemperatureUnit,
}

impl WeatherProvider {
    /// Create a new weather provider
    pub fn new(unit: TemperatureUnit) -> Result<Self, WeatherError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client: Arc::new(client),
            unit,
        })
    }

    /// Set temperature unit preference
    pub fn set_unit(&mut self, unit: TemperatureUnit) {
        self.unit = unit;
    }

    /// Fetch weather data for a location
    pub async fn fetch(&self, location: &Location) -> Result<WeatherData, WeatherError> {
        let unit_param = match self.unit {
            TemperatureUnit::Celsius | TemperatureUnit::Auto => "celsius",
            TemperatureUnit::Fahrenheit => "fahrenheit",
        };

        let url = format!(
            "{}?latitude={}&longitude={}&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code&daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_probability_max,sunrise,sunset&hourly=temperature_2m,weather_code,precipitation_probability&temperature_unit={}&wind_speed_unit=mph&forecast_days=7&timezone=auto",
            OPEN_METEO_BASE_URL,
            location.latitude,
            location.longitude,
            unit_param
        );

        tracing::debug!("Fetching weather from: {}", url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WeatherError::Parse(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let api_response: api::ForecastResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        self.parse_response(api_response, location)
    }

    fn parse_response(
        &self,
        resp: api::ForecastResponse,
        location: &Location,
    ) -> Result<WeatherData, WeatherError> {
        let now = Utc::now();

        let current = CurrentWeather {
            temperature: resp.current.temperature_2m,
            feels_like: resp.current.apparent_temperature,
            humidity: resp.current.relative_humidity_2m.clamp(0, 255) as u8,
            wind_speed: resp.current.wind_speed_10m,
            condition: WeatherCondition::from_wmo_code(resp.current.weather_code),
            updated_at: now,
        };

        let mut forecast = Vec::new();

        for (i, date_str) in resp.daily.time.iter().enumerate() {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| WeatherError::Parse(format!("Date parse error: {}", e)))?;

            let sunrise = Self::parse_time_from_datetime(&resp.daily.sunrise[i])?;
            let sunset = Self::parse_time_from_datetime(&resp.daily.sunset[i])?;

            // Collect hourly data for this day
            let hourly: Vec<HourlyForecast> = resp
                .hourly
                .time
                .iter()
                .enumerate()
                .filter(|(_, t)| t.starts_with(date_str))
                .map(|(j, t)| {
                    let time = Self::parse_time_from_datetime(t).unwrap_or_default();
                    HourlyForecast {
                        time,
                        temperature: resp.hourly.temperature_2m[j],
                        condition: WeatherCondition::from_wmo_code(resp.hourly.weather_code[j]),
                        precipitation_chance: resp.hourly.precipitation_probability[j].clamp(0, 100)
                            as u8,
                    }
                })
                .collect();

            forecast.push(DayForecast {
                date,
                high: resp.daily.temperature_2m_max[i],
                low: resp.daily.temperature_2m_min[i],
                condition: WeatherCondition::from_wmo_code(resp.daily.weather_code[i]),
                precipitation_chance: resp.daily.precipitation_probability_max[i].clamp(0, 100)
                    as u8,
                sunrise,
                sunset,
                hourly,
            });
        }

        Ok(WeatherData {
            current,
            forecast,
            location: location.clone(),
            fetched_at: now,
        })
    }

    fn parse_time_from_datetime(datetime_str: &str) -> Result<NaiveTime, WeatherError> {
        // Format: "2026-01-20T06:32"
        let time_part = datetime_str
            .split('T')
            .nth(1)
            .ok_or_else(|| WeatherError::Parse("Invalid datetime format".to_string()))?;

        NaiveTime::parse_from_str(time_part, "%H:%M")
            .map_err(|e| WeatherError::Parse(format!("Time parse error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_time_from_datetime() {
        let result = WeatherProvider::parse_time_from_datetime("2026-01-20T06:32").unwrap();
        assert_eq!(result.hour(), 6);
        assert_eq!(result.minute(), 32);
    }

    #[test]
    fn test_parse_time_invalid_format() {
        let result = WeatherProvider::parse_time_from_datetime("invalid");
        assert!(result.is_err());
    }

    // Integration test - requires network
    #[tokio::test]
    #[ignore] // Run with: cargo test -p myme-weather -- --ignored
    async fn test_fetch_real_api() {
        let provider = WeatherProvider::new(TemperatureUnit::Fahrenheit).unwrap();
        let location = Location {
            latitude: 47.6062,
            longitude: -122.3321,
            accuracy_meters: None,
            city_name: Some("Seattle".to_string()),
        };

        let result = provider.fetch(&location).await;
        assert!(result.is_ok());

        let data = result.unwrap();
        assert!(!data.forecast.is_empty());
        assert!(data.current.temperature != 0.0);
    }
}
