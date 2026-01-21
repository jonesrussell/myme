# Location & Weather Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add weather display to MyMe with sidebar footer widget, dashboard widget, and detailed forecast page using Open-Meteo API and system location services.

**Architecture:** New `myme-weather` crate handles location detection, API calls, and caching. The `WeatherModel` in `myme-ui` bridges Rust to QML. Three QML components provide the UI at different detail levels.

**Tech Stack:** Rust, Open-Meteo API (free, no key), chrono, reqwest, cxx-qt, Qt/QML, Kirigami

---

## Task 1: Create myme-weather Crate Scaffold

**Files:**
- Create: `crates/myme-weather/Cargo.toml`
- Create: `crates/myme-weather/src/lib.rs`
- Create: `crates/myme-weather/src/types.rs`
- Modify: `Cargo.toml` (root workspace)

**Step 1: Add crate to workspace**

Edit `Cargo.toml` at line 2-7:

```toml
[workspace]
members = [
    "crates/myme-core",
    "crates/myme-ui",
    "crates/myme-services",
    "crates/myme-auth",
    "crates/myme-integrations",
    "crates/myme-weather",
]
resolver = "2"
```

**Step 2: Create Cargo.toml for myme-weather**

Create `crates/myme-weather/Cargo.toml`:

```toml
[package]
name = "myme-weather"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
reqwest.workspace = true

# Date/time handling
chrono = { version = "0.4", features = ["serde"] }

# Platform-specific location
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Devices_Geolocation",
    "Foundation",
] }

[target.'cfg(target_os = "linux")'.dependencies]
zbus = { version = "4", default-features = false, features = ["tokio"] }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
wiremock = "0.6"
tempfile = "3"
```

**Step 3: Create lib.rs with module declarations**

Create `crates/myme-weather/src/lib.rs`:

```rust
//! Weather service for MyMe
//!
//! Provides weather data via Open-Meteo API with system location detection
//! and persistent caching.

pub mod types;
pub mod cache;
pub mod location;
pub mod provider;

pub use types::*;
pub use cache::WeatherCache;
pub use provider::WeatherProvider;
```

**Step 4: Create types.rs with data structures**

Create `crates/myme-weather/src/types.rs`:

```rust
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

/// Temperature unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    #[default]
    Auto,
    Celsius,
    Fahrenheit,
}

/// Weather condition categories mapped from WMO codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    #[default]
    Clear,
    PartlyCloudy,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    Snow,
    Sleet,
    Thunderstorm,
}

impl WeatherCondition {
    /// Convert WMO weather code to WeatherCondition
    /// See: https://open-meteo.com/en/docs#weathervariables
    pub fn from_wmo_code(code: i32) -> Self {
        match code {
            0 => Self::Clear,
            1..=2 => Self::PartlyCloudy,
            3 => Self::Cloudy,
            45 | 48 => Self::Fog,
            51 | 53 | 55 => Self::Drizzle,
            56 | 57 => Self::Sleet, // Freezing drizzle
            61 | 63 | 80 => Self::Rain,
            65 | 81 | 82 => Self::HeavyRain,
            66 | 67 => Self::Sleet, // Freezing rain
            71 | 73 | 75 | 77 | 85 | 86 => Self::Snow,
            95 | 96 | 99 => Self::Thunderstorm,
            _ => Self::Clear, // Unknown codes default to clear
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::PartlyCloudy => "Partly Cloudy",
            Self::Cloudy => "Cloudy",
            Self::Fog => "Fog",
            Self::Drizzle => "Drizzle",
            Self::Rain => "Rain",
            Self::HeavyRain => "Heavy Rain",
            Self::Snow => "Snow",
            Self::Sleet => "Sleet",
            Self::Thunderstorm => "Thunderstorm",
        }
    }

    /// Get icon name (Phosphor icon unicode will be in QML)
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Clear => "sun",
            Self::PartlyCloudy => "cloud_sun",
            Self::Cloudy => "cloud",
            Self::Fog => "cloud_fog",
            Self::Drizzle => "cloud_rain",
            Self::Rain => "cloud_rain",
            Self::HeavyRain => "cloud_rain",
            Self::Snow => "cloud_snow",
            Self::Sleet => "cloud_snow",
            Self::Thunderstorm => "cloud_lightning",
        }
    }
}

/// Geographic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub city_name: Option<String>,
}

/// Current weather conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentWeather {
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub condition: WeatherCondition,
    pub updated_at: DateTime<Utc>,
}

/// Hourly forecast entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub time: NaiveTime,
    pub temperature: f64,
    pub condition: WeatherCondition,
    pub precipitation_chance: u8,
}

/// Daily forecast entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayForecast {
    pub date: NaiveDate,
    pub high: f64,
    pub low: f64,
    pub condition: WeatherCondition,
    pub precipitation_chance: u8,
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
    pub hourly: Vec<HourlyForecast>,
}

/// Complete weather data bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub forecast: Vec<DayForecast>,
    pub location: Location,
    pub fetched_at: DateTime<Utc>,
}

/// Location service errors
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
    #[error("Location permission denied")]
    PermissionDenied,
    #[error("Location service unavailable")]
    ServiceUnavailable,
    #[error("Location request timed out")]
    Timeout,
    #[error("Location error: {0}")]
    Other(String),
}

/// Weather provider errors
#[derive(Debug, thiserror::Error)]
pub enum WeatherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Location error: {0}")]
    Location(#[from] LocationError),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Cache error: {0}")]
    Cache(String),
}
```

**Step 5: Verify crate compiles**

Run: `cargo build -p myme-weather`
Expected: Build succeeds (with warnings about unused modules)

**Step 6: Commit**

```bash
git add crates/myme-weather Cargo.toml
git commit -m "feat(weather): create myme-weather crate scaffold with types"
```

---

## Task 2: Implement WMO Code Mapping Tests

**Files:**
- Modify: `crates/myme-weather/src/types.rs`

**Step 1: Write tests for WMO code mapping**

Add to end of `crates/myme-weather/src/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmo_code_clear() {
        assert_eq!(WeatherCondition::from_wmo_code(0), WeatherCondition::Clear);
    }

    #[test]
    fn test_wmo_code_partly_cloudy() {
        assert_eq!(WeatherCondition::from_wmo_code(1), WeatherCondition::PartlyCloudy);
        assert_eq!(WeatherCondition::from_wmo_code(2), WeatherCondition::PartlyCloudy);
    }

    #[test]
    fn test_wmo_code_cloudy() {
        assert_eq!(WeatherCondition::from_wmo_code(3), WeatherCondition::Cloudy);
    }

    #[test]
    fn test_wmo_code_fog() {
        assert_eq!(WeatherCondition::from_wmo_code(45), WeatherCondition::Fog);
        assert_eq!(WeatherCondition::from_wmo_code(48), WeatherCondition::Fog);
    }

    #[test]
    fn test_wmo_code_drizzle() {
        assert_eq!(WeatherCondition::from_wmo_code(51), WeatherCondition::Drizzle);
        assert_eq!(WeatherCondition::from_wmo_code(53), WeatherCondition::Drizzle);
        assert_eq!(WeatherCondition::from_wmo_code(55), WeatherCondition::Drizzle);
    }

    #[test]
    fn test_wmo_code_rain() {
        assert_eq!(WeatherCondition::from_wmo_code(61), WeatherCondition::Rain);
        assert_eq!(WeatherCondition::from_wmo_code(63), WeatherCondition::Rain);
        assert_eq!(WeatherCondition::from_wmo_code(80), WeatherCondition::Rain);
    }

    #[test]
    fn test_wmo_code_heavy_rain() {
        assert_eq!(WeatherCondition::from_wmo_code(65), WeatherCondition::HeavyRain);
        assert_eq!(WeatherCondition::from_wmo_code(81), WeatherCondition::HeavyRain);
        assert_eq!(WeatherCondition::from_wmo_code(82), WeatherCondition::HeavyRain);
    }

    #[test]
    fn test_wmo_code_snow() {
        assert_eq!(WeatherCondition::from_wmo_code(71), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(73), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(75), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(77), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(85), WeatherCondition::Snow);
        assert_eq!(WeatherCondition::from_wmo_code(86), WeatherCondition::Snow);
    }

    #[test]
    fn test_wmo_code_sleet() {
        assert_eq!(WeatherCondition::from_wmo_code(56), WeatherCondition::Sleet);
        assert_eq!(WeatherCondition::from_wmo_code(57), WeatherCondition::Sleet);
        assert_eq!(WeatherCondition::from_wmo_code(66), WeatherCondition::Sleet);
        assert_eq!(WeatherCondition::from_wmo_code(67), WeatherCondition::Sleet);
    }

    #[test]
    fn test_wmo_code_thunderstorm() {
        assert_eq!(WeatherCondition::from_wmo_code(95), WeatherCondition::Thunderstorm);
        assert_eq!(WeatherCondition::from_wmo_code(96), WeatherCondition::Thunderstorm);
        assert_eq!(WeatherCondition::from_wmo_code(99), WeatherCondition::Thunderstorm);
    }

    #[test]
    fn test_wmo_code_unknown_defaults_to_clear() {
        assert_eq!(WeatherCondition::from_wmo_code(999), WeatherCondition::Clear);
        assert_eq!(WeatherCondition::from_wmo_code(-1), WeatherCondition::Clear);
    }

    #[test]
    fn test_condition_description() {
        assert_eq!(WeatherCondition::Clear.description(), "Clear");
        assert_eq!(WeatherCondition::Thunderstorm.description(), "Thunderstorm");
    }

    #[test]
    fn test_condition_icon_name() {
        assert_eq!(WeatherCondition::Clear.icon_name(), "sun");
        assert_eq!(WeatherCondition::Rain.icon_name(), "cloud_rain");
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p myme-weather`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/myme-weather/src/types.rs
git commit -m "test(weather): add WMO code mapping tests"
```

---

## Task 3: Implement Cache Layer

**Files:**
- Create: `crates/myme-weather/src/cache.rs`

**Step 1: Write failing test for cache staleness**

Create `crates/myme-weather/src/cache.rs`:

```rust
use crate::types::{WeatherData, WeatherError};
use chrono::{DateTime, Duration, Utc};
use std::path::PathBuf;

const STALE_THRESHOLD_MINUTES: i64 = 15;
const EXPIRED_THRESHOLD_HOURS: i64 = 2;

/// Persistent weather cache
#[derive(Debug)]
pub struct WeatherCache {
    cache_path: PathBuf,
    data: Option<WeatherData>,
}

impl WeatherCache {
    /// Create a new cache instance
    pub fn new(config_dir: &std::path::Path) -> Self {
        let cache_path = config_dir.join("weather_cache.json");
        Self {
            cache_path,
            data: None,
        }
    }

    /// Load cache from disk
    pub fn load(&mut self) -> Result<(), WeatherError> {
        if !self.cache_path.exists() {
            return Ok(());
        }

        let contents = std::fs::read_to_string(&self.cache_path)
            .map_err(|e| WeatherError::Cache(e.to_string()))?;

        self.data = serde_json::from_str(&contents)
            .map_err(|e| WeatherError::Cache(e.to_string()))?;

        Ok(())
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<(), WeatherError> {
        if let Some(data) = &self.data {
            if let Some(parent) = self.cache_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| WeatherError::Cache(e.to_string()))?;
            }

            let contents = serde_json::to_string_pretty(data)
                .map_err(|e| WeatherError::Cache(e.to_string()))?;

            std::fs::write(&self.cache_path, contents)
                .map_err(|e| WeatherError::Cache(e.to_string()))?;
        }
        Ok(())
    }

    /// Update cached data
    pub fn update(&mut self, data: WeatherData) {
        self.data = Some(data);
    }

    /// Get cached data if available
    pub fn get(&self) -> Option<&WeatherData> {
        self.data.as_ref()
    }

    /// Check if cache has data
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Check if cached data is stale (> 15 min old)
    pub fn is_stale(&self) -> bool {
        self.data
            .as_ref()
            .map(|d| {
                let age = Utc::now() - d.fetched_at;
                age > Duration::minutes(STALE_THRESHOLD_MINUTES)
            })
            .unwrap_or(true)
    }

    /// Check if cached data is expired (> 2 hours old)
    pub fn is_expired(&self) -> bool {
        self.data
            .as_ref()
            .map(|d| {
                let age = Utc::now() - d.fetched_at;
                age > Duration::hours(EXPIRED_THRESHOLD_HOURS)
            })
            .unwrap_or(true)
    }

    /// Get age of cached data in minutes
    pub fn age_minutes(&self) -> Option<i64> {
        self.data.as_ref().map(|d| {
            let age = Utc::now() - d.fetched_at;
            age.num_minutes()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn make_test_weather_data(fetched_at: DateTime<Utc>) -> WeatherData {
        WeatherData {
            current: CurrentWeather {
                temperature: 72.0,
                feels_like: 75.0,
                humidity: 45,
                wind_speed: 8.0,
                condition: WeatherCondition::Clear,
                updated_at: fetched_at,
            },
            forecast: vec![DayForecast {
                date: NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
                high: 78.0,
                low: 65.0,
                condition: WeatherCondition::Clear,
                precipitation_chance: 10,
                sunrise: chrono::NaiveTime::from_hms_opt(6, 32, 0).unwrap(),
                sunset: chrono::NaiveTime::from_hms_opt(20, 15, 0).unwrap(),
                hourly: vec![],
            }],
            location: Location {
                latitude: 47.6062,
                longitude: -122.3321,
                accuracy_meters: Some(100.0),
                city_name: Some("Seattle".to_string()),
            },
            fetched_at,
        }
    }

    #[test]
    fn test_cache_fresh_data_not_stale() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = WeatherCache::new(temp_dir.path());

        let data = make_test_weather_data(Utc::now());
        cache.update(data);

        assert!(!cache.is_stale());
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_cache_stale_after_15_minutes() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = WeatherCache::new(temp_dir.path());

        let old_time = Utc::now() - Duration::minutes(16);
        let data = make_test_weather_data(old_time);
        cache.update(data);

        assert!(cache.is_stale());
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_cache_expired_after_2_hours() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = WeatherCache::new(temp_dir.path());

        let old_time = Utc::now() - Duration::hours(3);
        let data = make_test_weather_data(old_time);
        cache.update(data);

        assert!(cache.is_stale());
        assert!(cache.is_expired());
    }

    #[test]
    fn test_cache_empty_is_stale_and_expired() {
        let temp_dir = TempDir::new().unwrap();
        let cache = WeatherCache::new(temp_dir.path());

        assert!(cache.is_stale());
        assert!(cache.is_expired());
        assert!(!cache.has_data());
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();

        // Save data
        {
            let mut cache = WeatherCache::new(temp_dir.path());
            let data = make_test_weather_data(Utc::now());
            cache.update(data);
            cache.save().unwrap();
        }

        // Load in new instance
        {
            let mut cache = WeatherCache::new(temp_dir.path());
            cache.load().unwrap();
            assert!(cache.has_data());
            assert_eq!(cache.get().unwrap().current.temperature, 72.0);
        }
    }

    #[test]
    fn test_cache_age_minutes() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = WeatherCache::new(temp_dir.path());

        let old_time = Utc::now() - Duration::minutes(10);
        let data = make_test_weather_data(old_time);
        cache.update(data);

        let age = cache.age_minutes().unwrap();
        assert!(age >= 10 && age <= 11);
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p myme-weather cache`
Expected: All cache tests pass

**Step 3: Commit**

```bash
git add crates/myme-weather/src/cache.rs
git commit -m "feat(weather): implement cache layer with staleness detection"
```

---

## Task 4: Implement Open-Meteo Provider

**Files:**
- Create: `crates/myme-weather/src/provider.rs`

**Step 1: Create provider with API response types**

Create `crates/myme-weather/src/provider.rs`:

```rust
use crate::types::*;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
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
```

**Step 2: Run tests**

Run: `cargo test -p myme-weather provider`
Expected: Unit tests pass (ignore integration test)

**Step 3: Commit**

```bash
git add crates/myme-weather/src/provider.rs
git commit -m "feat(weather): implement Open-Meteo API provider"
```

---

## Task 5: Implement Location Services

**Files:**
- Create: `crates/myme-weather/src/location.rs`

**Step 1: Create location module with platform abstraction**

Create `crates/myme-weather/src/location.rs`:

```rust
use crate::types::{Location, LocationError};

/// Get the current location from the system
pub async fn get_current_location() -> Result<Location, LocationError> {
    #[cfg(target_os = "windows")]
    {
        windows::get_location().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::get_location().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(LocationError::ServiceUnavailable)
    }
}

/// Check if location services are available
pub async fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows::is_available().await
    }

    #[cfg(target_os = "linux")]
    {
        linux::is_available().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::time::Duration;
    use windows::Devices::Geolocation::{
        GeolocationAccessStatus, Geolocator, PositionAccuracy,
    };

    pub async fn is_available() -> bool {
        match Geolocator::RequestAccessAsync() {
            Ok(op) => {
                match op.get() {
                    Ok(status) => status == GeolocationAccessStatus::Allowed,
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    pub async fn get_location() -> Result<Location, LocationError> {
        // Request access
        let access_status = Geolocator::RequestAccessAsync()
            .map_err(|e| LocationError::Other(e.to_string()))?
            .get()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        match access_status {
            GeolocationAccessStatus::Allowed => {}
            GeolocationAccessStatus::Denied => return Err(LocationError::PermissionDenied),
            _ => return Err(LocationError::ServiceUnavailable),
        }

        // Create geolocator
        let geolocator = Geolocator::new()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        geolocator
            .SetDesiredAccuracy(PositionAccuracy::Default)
            .map_err(|e| LocationError::Other(e.to_string()))?;

        // Get position with timeout
        let position = geolocator
            .GetGeopositionAsync()
            .map_err(|e| LocationError::Other(e.to_string()))?
            .get()
            .map_err(|e| {
                if e.to_string().contains("timeout") {
                    LocationError::Timeout
                } else {
                    LocationError::Other(e.to_string())
                }
            })?;

        let coord = position
            .Coordinate()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let point = coord
            .Point()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let pos = point
            .Position()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let accuracy = coord
            .Accuracy()
            .ok()
            .and_then(|a| a.GetDouble().ok());

        Ok(Location {
            latitude: pos.Latitude,
            longitude: pos.Longitude,
            accuracy_meters: accuracy,
            city_name: None, // Would need reverse geocoding
        })
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use zbus::Connection;

    const GEOCLUE_BUS: &str = "org.freedesktop.GeoClue2";
    const GEOCLUE_MANAGER_PATH: &str = "/org/freedesktop/GeoClue2/Manager";

    pub async fn is_available() -> bool {
        match Connection::system().await {
            Ok(conn) => {
                conn.call_method(
                    Some(GEOCLUE_BUS),
                    GEOCLUE_MANAGER_PATH,
                    Some("org.freedesktop.DBus.Peer"),
                    "Ping",
                    &(),
                )
                .await
                .is_ok()
            }
            Err(_) => false,
        }
    }

    pub async fn get_location() -> Result<Location, LocationError> {
        let conn = Connection::system()
            .await
            .map_err(|_| LocationError::ServiceUnavailable)?;

        // Create a client
        let reply: zbus::Message = conn
            .call_method(
                Some(GEOCLUE_BUS),
                GEOCLUE_MANAGER_PATH,
                Some("org.freedesktop.GeoClue2.Manager"),
                "CreateClient",
                &(),
            )
            .await
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let client_path: zbus::zvariant::OwnedObjectPath = reply
            .body()
            .deserialize()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        // Set desktop ID (required)
        conn.call_method(
            Some(GEOCLUE_BUS),
            client_path.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Set",
            &(
                "org.freedesktop.GeoClue2.Client",
                "DesktopId",
                zbus::zvariant::Value::from("myme"),
            ),
        )
        .await
        .map_err(|e| LocationError::Other(e.to_string()))?;

        // Start the client
        conn.call_method(
            Some(GEOCLUE_BUS),
            client_path.as_str(),
            Some("org.freedesktop.GeoClue2.Client"),
            "Start",
            &(),
        )
        .await
        .map_err(|e| LocationError::Other(e.to_string()))?;

        // Get location path
        let location_reply: zbus::Message = conn
            .call_method(
                Some(GEOCLUE_BUS),
                client_path.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.GeoClue2.Client", "Location"),
            )
            .await
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let location_path: zbus::zvariant::OwnedObjectPath = location_reply
            .body()
            .deserialize::<zbus::zvariant::Value>()
            .map_err(|e| LocationError::Other(e.to_string()))?
            .downcast_ref::<zbus::zvariant::ObjectPath>()
            .ok_or_else(|| LocationError::Other("Invalid location path".to_string()))?
            .to_owned()
            .into();

        // Get latitude
        let lat_reply: zbus::Message = conn
            .call_method(
                Some(GEOCLUE_BUS),
                location_path.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.GeoClue2.Location", "Latitude"),
            )
            .await
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let latitude: f64 = lat_reply
            .body()
            .deserialize::<zbus::zvariant::Value>()
            .map_err(|e| LocationError::Other(e.to_string()))?
            .downcast_ref::<f64>()
            .copied()
            .ok_or_else(|| LocationError::Other("Invalid latitude".to_string()))?;

        // Get longitude
        let lon_reply: zbus::Message = conn
            .call_method(
                Some(GEOCLUE_BUS),
                location_path.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.GeoClue2.Location", "Longitude"),
            )
            .await
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let longitude: f64 = lon_reply
            .body()
            .deserialize::<zbus::zvariant::Value>()
            .map_err(|e| LocationError::Other(e.to_string()))?
            .downcast_ref::<f64>()
            .copied()
            .ok_or_else(|| LocationError::Other("Invalid longitude".to_string()))?;

        // Stop the client
        let _ = conn
            .call_method(
                Some(GEOCLUE_BUS),
                client_path.as_str(),
                Some("org.freedesktop.GeoClue2.Client"),
                "Stop",
                &(),
            )
            .await;

        Ok(Location {
            latitude,
            longitude,
            accuracy_meters: None,
            city_name: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require actual location services, so they're ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_location_available() {
        let available = is_available().await;
        println!("Location available: {}", available);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_location() {
        let result = get_current_location().await;
        match result {
            Ok(loc) => {
                println!("Location: {:?}", loc);
                assert!(loc.latitude != 0.0);
                assert!(loc.longitude != 0.0);
            }
            Err(e) => {
                println!("Location error (may be expected): {:?}", e);
            }
        }
    }
}
```

**Step 2: Verify compilation on target platform**

Run: `cargo build -p myme-weather`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/myme-weather/src/location.rs
git commit -m "feat(weather): implement platform location services (Windows/Linux)"
```

---

## Task 6: Add Weather Config to myme-core

**Files:**
- Modify: `crates/myme-core/src/config.rs`

**Step 1: Add weather config section**

Add to `crates/myme-core/src/config.rs` after line 36 (after UiConfig):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Temperature unit: "auto", "celsius", "fahrenheit"
    pub temperature_unit: String,

    /// Refresh interval in minutes
    pub refresh_interval_minutes: u32,

    /// Enable weather feature
    pub enabled: bool,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            temperature_unit: "auto".to_string(),
            refresh_interval_minutes: 15,
            enabled: true,
        }
    }
}
```

**Step 2: Add weather field to Config struct**

Modify the Config struct around line 5-15:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application configuration directory
    pub config_dir: PathBuf,

    /// Service configurations
    pub services: ServiceConfig,

    /// UI preferences
    pub ui: UiConfig,

    /// Weather settings
    #[serde(default)]
    pub weather: WeatherConfig,
}
```

**Step 3: Update Config::default()**

Update the Default impl around line 38-56:

```rust
impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("myme");

        Self {
            config_dir,
            services: ServiceConfig {
                todo_api_url: "http://localhost:8008".to_string(),
                jwt_token: std::env::var("GODO_JWT_TOKEN").ok(),
            },
            ui: UiConfig {
                window_width: 1200,
                window_height: 800,
                dark_mode: false,
            },
            weather: WeatherConfig::default(),
        }
    }
}
```

**Step 4: Verify build**

Run: `cargo build -p myme-core`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/myme-core/src/config.rs
git commit -m "feat(config): add weather configuration section"
```

---

## Task 7: Create WeatherModel Bridge

**Files:**
- Create: `crates/myme-ui/src/models/weather_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/Cargo.toml`

**Step 1: Add myme-weather dependency**

Add to `crates/myme-ui/Cargo.toml` after line 21:

```toml
myme-weather = { path = "../myme-weather" }
```

**Step 2: Create weather_model.rs**

Create `crates/myme-ui/src/models/weather_model.rs`:

```rust
use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_weather::{WeatherCache, WeatherCondition, WeatherData, WeatherProvider, TemperatureUnit};
use std::sync::Arc;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, temperature)]
        #[qproperty(QString, condition_text)]
        #[qproperty(QString, condition_icon)]
        #[qproperty(f64, feels_like)]
        #[qproperty(i32, humidity)]
        #[qproperty(f64, wind_speed)]
        #[qproperty(f64, high_temp)]
        #[qproperty(f64, low_temp)]
        #[qproperty(QString, sunrise)]
        #[qproperty(QString, sunset)]
        #[qproperty(QString, location_name)]
        #[qproperty(bool, is_stale)]
        #[qproperty(bool, is_loading)]
        #[qproperty(bool, has_data)]
        #[qproperty(QString, error_message)]
        #[qproperty(i32, stale_minutes)]
        type WeatherModel = super::WeatherModelRust;

        #[qinvokable]
        fn refresh(self: Pin<&mut WeatherModel>);

        #[qinvokable]
        fn forecast_count(self: &WeatherModel) -> i32;

        #[qinvokable]
        fn get_forecast_date(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_high(self: &WeatherModel, index: i32) -> f64;

        #[qinvokable]
        fn get_forecast_low(self: &WeatherModel, index: i32) -> f64;

        #[qinvokable]
        fn get_forecast_condition(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_icon(self: &WeatherModel, index: i32) -> QString;

        #[qinvokable]
        fn get_forecast_precip(self: &WeatherModel, index: i32) -> i32;

        #[qinvokable]
        fn hourly_count(self: &WeatherModel, day_index: i32) -> i32;

        #[qinvokable]
        fn get_hourly_time(self: &WeatherModel, day_index: i32, hour_index: i32) -> QString;

        #[qinvokable]
        fn get_hourly_temp(self: &WeatherModel, day_index: i32, hour_index: i32) -> f64;

        #[qinvokable]
        fn get_hourly_icon(self: &WeatherModel, day_index: i32, hour_index: i32) -> QString;

        #[qsignal]
        fn weather_updated(self: Pin<&mut WeatherModel>);
    }
}

#[derive(Default)]
pub struct WeatherModelRust {
    // Properties
    temperature: f64,
    condition_text: QString,
    condition_icon: QString,
    feels_like: f64,
    humidity: i32,
    wind_speed: f64,
    high_temp: f64,
    low_temp: f64,
    sunrise: QString,
    sunset: QString,
    location_name: QString,
    is_stale: bool,
    is_loading: bool,
    has_data: bool,
    error_message: QString,
    stale_minutes: i32,

    // Internal state
    weather_data: Option<WeatherData>,
    cache: Option<WeatherCache>,
    provider: Option<WeatherProvider>,
    runtime: Option<tokio::runtime::Handle>,
}

impl WeatherModelRust {
    pub fn initialize(&mut self, config_dir: &std::path::Path, runtime: tokio::runtime::Handle) {
        self.cache = Some(WeatherCache::new(config_dir));
        self.provider = Some(WeatherProvider::new(TemperatureUnit::Auto).unwrap());
        self.runtime = Some(runtime);

        // Load cached data
        if let Some(cache) = &mut self.cache {
            if cache.load().is_ok() {
                if let Some(data) = cache.get() {
                    self.update_from_data(data.clone());
                }
            }
        }
    }

    fn update_from_data(&mut self, data: WeatherData) {
        self.temperature = data.current.temperature;
        self.condition_text = QString::from(data.current.condition.description());
        self.condition_icon = QString::from(Self::condition_to_icon(&data.current.condition));
        self.feels_like = data.current.feels_like;
        self.humidity = data.current.humidity as i32;
        self.wind_speed = data.current.wind_speed;

        if let Some(today) = data.forecast.first() {
            self.high_temp = today.high;
            self.low_temp = today.low;
            self.sunrise = QString::from(today.sunrise.format("%I:%M %p").to_string());
            self.sunset = QString::from(today.sunset.format("%I:%M %p").to_string());
        }

        if let Some(name) = &data.location.city_name {
            self.location_name = QString::from(name.as_str());
        }

        self.has_data = true;
        self.weather_data = Some(data);
    }

    fn condition_to_icon(condition: &WeatherCondition) -> &'static str {
        // Return Phosphor icon unicode for QML
        match condition {
            WeatherCondition::Clear => "\u{ed3e}",           // sun
            WeatherCondition::PartlyCloudy => "\u{ea5a}",    // cloud-sun
            WeatherCondition::Cloudy => "\u{ea52}",          // cloud
            WeatherCondition::Fog => "\u{ea56}",             // cloud-fog
            WeatherCondition::Drizzle => "\u{ea58}",         // cloud-rain
            WeatherCondition::Rain => "\u{ea58}",            // cloud-rain
            WeatherCondition::HeavyRain => "\u{ea58}",       // cloud-rain
            WeatherCondition::Snow => "\u{ea5c}",            // cloud-snow
            WeatherCondition::Sleet => "\u{ea5c}",           // cloud-snow
            WeatherCondition::Thunderstorm => "\u{ea54}",    // cloud-lightning
        }
    }

    fn get_forecast(&self, index: i32) -> Option<&myme_weather::DayForecast> {
        self.weather_data
            .as_ref()
            .and_then(|d| d.forecast.get(index as usize))
    }

    fn get_hourly(&self, day_index: i32, hour_index: i32) -> Option<&myme_weather::HourlyForecast> {
        self.get_forecast(day_index)
            .and_then(|d| d.hourly.get(hour_index as usize))
    }
}

impl qobject::WeatherModel {
    pub fn refresh(mut self: Pin<&mut Self>) {
        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Weather not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        self.as_mut().set_is_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Try to get location and fetch weather
        match runtime.block_on(async {
            let location = myme_weather::location::get_current_location().await?;
            provider.fetch(&location).await
        }) {
            Ok(data) => {
                tracing::info!("Weather fetched successfully");
                self.as_mut().rust_mut().update_from_data(data.clone());

                // Update cache
                if let Some(cache) = &mut self.as_mut().rust_mut().cache {
                    cache.update(data);
                    let _ = cache.save();
                }

                self.as_mut().set_is_stale(false);
                self.as_mut().set_stale_minutes(0);
                self.as_mut().set_is_loading(false);
                self.weather_updated();
            }
            Err(e) => {
                tracing::error!("Weather fetch failed: {}", e);
                self.as_mut().set_error_message(QString::from(format!("Failed: {}", e)));
                self.as_mut().set_is_loading(false);

                // Check cache staleness
                if let Some(cache) = &self.as_ref().rust().cache {
                    self.as_mut().set_is_stale(cache.is_stale());
                    if let Some(age) = cache.age_minutes() {
                        self.as_mut().set_stale_minutes(age as i32);
                    }
                }
            }
        }
    }

    pub fn forecast_count(&self) -> i32 {
        self.rust()
            .weather_data
            .as_ref()
            .map(|d| d.forecast.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_forecast_date(&self, index: i32) -> QString {
        self.rust()
            .get_forecast(index)
            .map(|f| QString::from(f.date.format("%a, %b %d").to_string()))
            .unwrap_or_default()
    }

    pub fn get_forecast_high(&self, index: i32) -> f64 {
        self.rust().get_forecast(index).map(|f| f.high).unwrap_or(0.0)
    }

    pub fn get_forecast_low(&self, index: i32) -> f64 {
        self.rust().get_forecast(index).map(|f| f.low).unwrap_or(0.0)
    }

    pub fn get_forecast_condition(&self, index: i32) -> QString {
        self.rust()
            .get_forecast(index)
            .map(|f| QString::from(f.condition.description()))
            .unwrap_or_default()
    }

    pub fn get_forecast_icon(&self, index: i32) -> QString {
        self.rust()
            .get_forecast(index)
            .map(|f| QString::from(WeatherModelRust::condition_to_icon(&f.condition)))
            .unwrap_or_default()
    }

    pub fn get_forecast_precip(&self, index: i32) -> i32 {
        self.rust()
            .get_forecast(index)
            .map(|f| f.precipitation_chance as i32)
            .unwrap_or(0)
    }

    pub fn hourly_count(&self, day_index: i32) -> i32 {
        self.rust()
            .get_forecast(day_index)
            .map(|f| f.hourly.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_hourly_time(&self, day_index: i32, hour_index: i32) -> QString {
        self.rust()
            .get_hourly(day_index, hour_index)
            .map(|h| QString::from(h.time.format("%I %p").to_string()))
            .unwrap_or_default()
    }

    pub fn get_hourly_temp(&self, day_index: i32, hour_index: i32) -> f64 {
        self.rust()
            .get_hourly(day_index, hour_index)
            .map(|h| h.temperature)
            .unwrap_or(0.0)
    }

    pub fn get_hourly_icon(&self, day_index: i32, hour_index: i32) -> QString {
        self.rust()
            .get_hourly(day_index, hour_index)
            .map(|h| QString::from(WeatherModelRust::condition_to_icon(&h.condition)))
            .unwrap_or_default()
    }
}
```

**Step 3: Export weather_model from mod.rs**

Edit `crates/myme-ui/src/models/mod.rs`:

```rust
pub mod jwt_model;
pub mod note_model;
pub mod repo_model;
pub mod weather_model;
```

**Step 4: Verify build**

Run: `cargo build -p myme-ui`
Expected: Compiles (may have warnings)

**Step 5: Commit**

```bash
git add crates/myme-ui/src/models/weather_model.rs crates/myme-ui/src/models/mod.rs crates/myme-ui/Cargo.toml
git commit -m "feat(ui): create WeatherModel cxx-qt bridge"
```

---

## Task 8: Add Weather Icons to Icons.qml

**Files:**
- Modify: `crates/myme-ui/qml/Icons.qml`

**Step 1: Add weather icons**

Add after line 58 (after circleHalf) in `crates/myme-ui/qml/Icons.qml`:

```qml
    // Weather
    readonly property string cloud: "\uea52"
    readonly property string cloudSun: "\uea5a"
    readonly property string cloudRain: "\uea58"
    readonly property string cloudSnow: "\uea5c"
    readonly property string cloudLightning: "\uea54"
    readonly property string cloudFog: "\uea56"
    readonly property string thermometer: "\ued57"
    readonly property string drop: "\ueab1"
    readonly property string wind: "\uedc7"
    readonly property string clock: "\uea42"
    readonly property string mapPin: "\uebe3"
```

**Step 2: Commit**

```bash
git add crates/myme-ui/qml/Icons.qml
git commit -m "feat(ui): add weather icons to Icons.qml"
```

---

## Task 9: Create WeatherCompact Component

**Files:**
- Create: `crates/myme-ui/qml/components/WeatherCompact.qml`
- Modify: `qml.qrc`

**Step 1: Create components directory and WeatherCompact.qml**

Create `crates/myme-ui/qml/components/WeatherCompact.qml`:

```qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: weatherCompact

    property bool expanded: true
    property real temp: 72
    property string condition: "Sunny"
    property string icon: Icons.sun
    property bool isStale: false
    property bool isLoading: false
    property bool hasError: false

    signal clicked()

    Layout.fillWidth: true
    Layout.preferredHeight: 44
    radius: Theme.buttonRadius
    color: mouseArea.containsMouse ? Theme.sidebarHover : "transparent"

    Behavior on color {
        ColorAnimation { duration: 100 }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: weatherCompact.clicked()
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacingSm
        anchors.rightMargin: Theme.spacingSm
        spacing: Theme.spacingSm

        // Weather icon or loading spinner
        Label {
            text: isLoading ? Icons.spinner : (hasError ? Icons.warning : icon)
            font.family: Icons.family
            font.pixelSize: 18
            color: hasError ? Theme.warning : Theme.text
            opacity: isStale ? 0.7 : 1.0

            RotationAnimation on rotation {
                running: isLoading
                loops: Animation.Infinite
                from: 0
                to: 360
                duration: 1000
            }
        }

        // Temperature
        Label {
            visible: !hasError
            text: Math.round(temp) + "°"
            font.pixelSize: Theme.fontSizeNormal
            font.bold: true
            color: Theme.text
            opacity: isStale ? 0.7 : 1.0
        }

        // Condition text (only when expanded)
        Label {
            visible: expanded && !hasError
            text: condition
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.textSecondary
            opacity: isStale ? 0.7 : 1.0
            Layout.fillWidth: true
            elide: Text.ElideRight

            Behavior on opacity {
                NumberAnimation { duration: 150 }
            }
        }

        // Error dash
        Label {
            visible: hasError
            text: "—"
            font.pixelSize: Theme.fontSizeNormal
            color: Theme.textSecondary
        }

        // Stale indicator
        Label {
            visible: isStale && !isLoading && !hasError
            text: Icons.clock
            font.family: Icons.family
            font.pixelSize: 12
            color: Theme.textMuted
        }

        Item {
            Layout.fillWidth: !expanded
        }
    }

    ToolTip.visible: mouseArea.containsMouse
    ToolTip.text: hasError ? "Weather unavailable" : (isStale ? "Weather data is stale" : condition + " " + Math.round(temp) + "°")
    ToolTip.delay: 500
}
```

**Step 2: Update qml.qrc**

Add after line 13 in `qml.qrc`:

```xml
        <file>crates/myme-ui/qml/components/WeatherCompact.qml</file>
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/components/WeatherCompact.qml qml.qrc
git commit -m "feat(ui): create WeatherCompact sidebar component"
```

---

## Task 10: Create WeatherWidget Component

**Files:**
- Create: `crates/myme-ui/qml/components/WeatherWidget.qml`
- Modify: `qml.qrc`

**Step 1: Create WeatherWidget.qml**

Create `crates/myme-ui/qml/components/WeatherWidget.qml`:

```qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: weatherWidget

    property real temp: 72
    property string condition: "Sunny"
    property string icon: Icons.sun
    property real feelsLike: 75
    property real high: 78
    property real low: 65
    property int humidity: 45
    property real windSpeed: 8
    property string sunrise: "6:32 AM"
    property string sunset: "8:15 PM"
    property string locationName: ""
    property bool isStale: false
    property bool isLoading: false
    property bool hasError: false
    property string errorMessage: ""

    signal clicked()

    implicitWidth: 300
    implicitHeight: contentLayout.implicitHeight + Theme.spacingMd * 2
    radius: Theme.cardRadius
    color: Theme.surface
    border.color: mouseArea.containsMouse ? Theme.primary : Theme.border
    border.width: 1

    Behavior on border.color {
        ColorAnimation { duration: 100 }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: weatherWidget.clicked()
    }

    ColumnLayout {
        id: contentLayout
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingSm

        // Loading state
        Item {
            visible: isLoading
            Layout.fillWidth: true
            Layout.preferredHeight: 100
            Layout.alignment: Qt.AlignCenter

            Column {
                anchors.centerIn: parent
                spacing: Theme.spacingSm

                Label {
                    text: Icons.spinner
                    font.family: Icons.family
                    font.pixelSize: 32
                    color: Theme.textSecondary
                    anchors.horizontalCenter: parent.horizontalCenter

                    RotationAnimation on rotation {
                        running: isLoading
                        loops: Animation.Infinite
                        from: 0
                        to: 360
                        duration: 1000
                    }
                }

                Label {
                    text: "Loading weather..."
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                }
            }
        }

        // Error state
        Item {
            visible: hasError && !isLoading
            Layout.fillWidth: true
            Layout.preferredHeight: 100

            Column {
                anchors.centerIn: parent
                spacing: Theme.spacingSm

                Label {
                    text: Icons.warning
                    font.family: Icons.family
                    font.pixelSize: 32
                    color: Theme.warning
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Label {
                    text: errorMessage || "Weather unavailable"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                    width: parent.width
                }
            }
        }

        // Normal content
        ColumnLayout {
            visible: !isLoading && !hasError
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            // Location (if available)
            Label {
                visible: locationName !== ""
                text: locationName
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
            }

            // Main weather row
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingMd

                Label {
                    text: icon
                    font.family: Icons.family
                    font.pixelSize: 40
                    color: Theme.text
                    opacity: isStale ? 0.7 : 1.0
                }

                Column {
                    spacing: 2

                    RowLayout {
                        spacing: Theme.spacingSm

                        Label {
                            text: Math.round(temp) + "°"
                            font.pixelSize: Theme.fontSizeXLarge
                            font.bold: true
                            color: Theme.text
                        }

                        Label {
                            text: condition
                            font.pixelSize: Theme.fontSizeMedium
                            color: Theme.textSecondary
                        }
                    }

                    Label {
                        text: "Feels like " + Math.round(feelsLike) + "° • H:" + Math.round(high) + "° L:" + Math.round(low) + "°"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }
            }

            // Details row
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingLg

                RowLayout {
                    spacing: Theme.spacingXs

                    Label {
                        text: Icons.drop
                        font.family: Icons.family
                        font.pixelSize: 14
                        color: Theme.textSecondary
                    }

                    Label {
                        text: humidity + "%"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }

                RowLayout {
                    spacing: Theme.spacingXs

                    Label {
                        text: Icons.wind
                        font.family: Icons.family
                        font.pixelSize: 14
                        color: Theme.textSecondary
                    }

                    Label {
                        text: Math.round(windSpeed) + " mph"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }
            }

            // Sunrise/Sunset row
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingLg

                RowLayout {
                    spacing: Theme.spacingXs

                    Label {
                        text: Icons.sun
                        font.family: Icons.family
                        font.pixelSize: 14
                        color: "#f59e0b"
                    }

                    Label {
                        text: sunrise
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }

                RowLayout {
                    spacing: Theme.spacingXs

                    Label {
                        text: Icons.moon
                        font.family: Icons.family
                        font.pixelSize: 14
                        color: "#a5b4fc"
                    }

                    Label {
                        text: sunset
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }
            }

            // Stale indicator
            RowLayout {
                visible: isStale
                spacing: Theme.spacingXs

                Label {
                    text: Icons.clock
                    font.family: Icons.family
                    font.pixelSize: 12
                    color: Theme.textMuted
                }

                Label {
                    text: "Data may be outdated"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textMuted
                }
            }
        }
    }
}
```

**Step 2: Update qml.qrc**

Add after the WeatherCompact line:

```xml
        <file>crates/myme-ui/qml/components/WeatherWidget.qml</file>
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/components/WeatherWidget.qml qml.qrc
git commit -m "feat(ui): create WeatherWidget dashboard component"
```

---

## Task 11: Create WeatherPage

**Files:**
- Create: `crates/myme-ui/qml/pages/WeatherPage.qml`
- Modify: `qml.qrc`

**Step 1: Create WeatherPage.qml**

Create `crates/myme-ui/qml/pages/WeatherPage.qml`:

```qml
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."
import "../components"

Page {
    id: weatherPage
    title: "Weather"

    property var weatherModel: null
    property int expandedDay: -1

    background: Rectangle {
        color: Theme.background
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Weather"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }

            Button {
                text: Icons.arrowsClockwise
                font.family: Icons.family
                font.pixelSize: 18
                flat: true
                onClicked: {
                    if (weatherModel) {
                        weatherModel.refresh()
                    }
                }

                ToolTip.visible: hovered
                ToolTip.text: "Refresh weather"
            }
        }
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingLg

            // Current conditions widget
            WeatherWidget {
                Layout.fillWidth: true
                Layout.maximumWidth: 400

                temp: weatherModel ? weatherModel.temperature : 0
                condition: weatherModel ? weatherModel.condition_text : ""
                icon: weatherModel ? weatherModel.condition_icon : Icons.sun
                feelsLike: weatherModel ? weatherModel.feels_like : 0
                high: weatherModel ? weatherModel.high_temp : 0
                low: weatherModel ? weatherModel.low_temp : 0
                humidity: weatherModel ? weatherModel.humidity : 0
                windSpeed: weatherModel ? weatherModel.wind_speed : 0
                sunrise: weatherModel ? weatherModel.sunrise : ""
                sunset: weatherModel ? weatherModel.sunset : ""
                locationName: weatherModel ? weatherModel.location_name : ""
                isStale: weatherModel ? weatherModel.is_stale : false
                isLoading: weatherModel ? weatherModel.is_loading : false
                hasError: weatherModel ? !weatherModel.has_data && weatherModel.error_message !== "" : false
                errorMessage: weatherModel ? weatherModel.error_message : ""
            }

            // 7-day forecast header
            Label {
                visible: weatherModel && weatherModel.forecast_count() > 0
                text: "7-Day Forecast"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
                Layout.topMargin: Theme.spacingSm
            }

            // Forecast list
            Repeater {
                model: weatherModel ? weatherModel.forecast_count() : 0

                delegate: Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: expandedDay === index ? forecastContent.implicitHeight + Theme.spacingMd * 2 : 60
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: forecastMouse.containsMouse ? Theme.primary : Theme.border
                    border.width: 1

                    Behavior on Layout.preferredHeight {
                        NumberAnimation { duration: 200; easing.type: Easing.OutQuad }
                    }

                    MouseArea {
                        id: forecastMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: expandedDay = (expandedDay === index) ? -1 : index
                    }

                    ColumnLayout {
                        id: forecastContent
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        // Summary row
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            Label {
                                text: weatherModel.get_forecast_date(index)
                                font.pixelSize: Theme.fontSizeNormal
                                font.bold: true
                                color: Theme.text
                                Layout.preferredWidth: 100
                            }

                            Label {
                                text: weatherModel.get_forecast_icon(index)
                                font.family: Icons.family
                                font.pixelSize: 20
                                color: Theme.text
                            }

                            Label {
                                text: weatherModel.get_forecast_condition(index)
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                                Layout.fillWidth: true
                            }

                            Label {
                                text: Math.round(weatherModel.get_forecast_high(index)) + "°"
                                font.pixelSize: Theme.fontSizeNormal
                                font.bold: true
                                color: Theme.text
                            }

                            Label {
                                text: Math.round(weatherModel.get_forecast_low(index)) + "°"
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textSecondary
                            }

                            Label {
                                text: Icons.drop
                                font.family: Icons.family
                                font.pixelSize: 14
                                color: Theme.info
                            }

                            Label {
                                text: weatherModel.get_forecast_precip(index) + "%"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                                Layout.preferredWidth: 40
                            }

                            Label {
                                text: expandedDay === index ? Icons.caretUp : Icons.caretDown
                                font.family: Icons.family
                                font.pixelSize: 16
                                color: Theme.textSecondary
                            }
                        }

                        // Hourly forecast (expanded)
                        Flow {
                            visible: expandedDay === index
                            Layout.fillWidth: true
                            spacing: Theme.spacingSm

                            Repeater {
                                model: expandedDay === index ? weatherModel.hourly_count(index) : 0

                                delegate: Rectangle {
                                    width: 50
                                    height: 70
                                    radius: 4
                                    color: Theme.surfaceAlt

                                    Column {
                                        anchors.centerIn: parent
                                        spacing: 2

                                        Label {
                                            text: weatherModel.get_hourly_time(index, modelData)
                                            font.pixelSize: 10
                                            color: Theme.textMuted
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }

                                        Label {
                                            text: weatherModel.get_hourly_icon(index, modelData)
                                            font.family: Icons.family
                                            font.pixelSize: 16
                                            color: Theme.text
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }

                                        Label {
                                            text: Math.round(weatherModel.get_hourly_temp(index, modelData)) + "°"
                                            font.pixelSize: Theme.fontSizeSmall
                                            font.bold: true
                                            color: Theme.text
                                            anchors.horizontalCenter: parent.horizontalCenter
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Item {
                Layout.fillHeight: true
            }
        }
    }
}
```

**Step 2: Update qml.qrc**

Add after SettingsPage.qml line:

```xml
        <file>crates/myme-ui/qml/pages/WeatherPage.qml</file>
```

**Step 3: Commit**

```bash
git add crates/myme-ui/qml/pages/WeatherPage.qml qml.qrc
git commit -m "feat(ui): create WeatherPage with forecast view"
```

---

## Task 12: Integrate Weather into Main.qml

**Files:**
- Modify: `crates/myme-ui/qml/Main.qml`

**Step 1: Add weather to navigation model**

In Main.qml around line 143, add weather to the nav model:

```qml
                Repeater {
                    model: [
                        {
                            id: "notes",
                            icon: Icons.notePencil,
                            label: "Notes",
                            enabled: true
                        },
                        {
                            id: "weather",
                            icon: Icons.sun,
                            label: "Weather",
                            enabled: true
                        },
                        {
                            id: "repos",
                            icon: Icons.folderSimple,
                            label: "Repos",
                            enabled: false
                        },
                        {
                            id: "devtools",
                            icon: Icons.wrench,
                            label: "Dev Tools",
                            enabled: true
                        }
                    ]
```

**Step 2: Add weather page navigation**

Around line 185-190, update the onClicked handler:

```qml
                            onClicked: {
                                if (modelData.enabled) {
                                    currentPage = modelData.id;
                                    if (modelData.id === "notes")
                                        stackView.replace("pages/NotePage.qml");
                                    else if (modelData.id === "weather")
                                        stackView.replace("pages/WeatherPage.qml");
                                    else if (modelData.id === "repos")
                                        stackView.replace("pages/RepoPage.qml");
                                    else if (modelData.id === "devtools")
                                        stackView.replace("pages/DevToolsPage.qml");
                                }
                            }
```

**Step 3: Add WeatherCompact to sidebar footer**

Before the Settings button (around line 237), add:

```qml
                // Weather compact widget
                WeatherCompact {
                    id: weatherCompact
                    expanded: !sidebarCollapsed
                    // TODO: Connect to WeatherModel
                    temp: 72
                    condition: "Sunny"
                    icon: Icons.sun
                    isStale: false
                    isLoading: false
                    hasError: false

                    onClicked: {
                        currentPage = "weather";
                        stackView.replace("pages/WeatherPage.qml");
                    }
                }
```

Also add the import at the top:

```qml
import "components"
```

**Step 4: Commit**

```bash
git add crates/myme-ui/qml/Main.qml
git commit -m "feat(ui): integrate weather into Main.qml navigation and sidebar"
```

---

## Task 13: Add Weather Settings Section

**Files:**
- Modify: `crates/myme-ui/qml/pages/SettingsPage.qml`

**Step 1: Add weather settings section**

Add after the Appearance section (around line 255):

```qml
            // Weather Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: weatherContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: weatherContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: "Weather"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                    }

                    Label {
                        text: "Configure weather display preferences and location settings."
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Temperature unit selection
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingMd

                        Label {
                            text: "Temperature Unit"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            Layout.preferredWidth: 150
                        }

                        ComboBox {
                            id: tempUnitCombo
                            model: ["Auto (System)", "Celsius", "Fahrenheit"]
                            currentIndex: 0
                            Layout.preferredWidth: 160

                            background: Rectangle {
                                implicitWidth: 160
                                implicitHeight: 36
                                color: Theme.inputBg
                                border.color: tempUnitCombo.pressed ? Theme.primary : Theme.inputBorder
                                border.width: 1
                                radius: Theme.inputRadius
                            }

                            contentItem: Text {
                                leftPadding: Theme.spacingSm
                                text: tempUnitCombo.displayText
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.text
                                verticalAlignment: Text.AlignVCenter
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Theme.border
                    }

                    // Location status
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingMd

                        Label {
                            text: "Location"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            Layout.preferredWidth: 150
                        }

                        Label {
                            text: Icons.mapPin
                            font.family: Icons.family
                            font.pixelSize: 16
                            color: Theme.success
                        }

                        Label {
                            text: "Detected"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textSecondary
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "Refresh"
                            flat: true
                            font.pixelSize: Theme.fontSizeSmall
                        }
                    }

                    // Location permission hint
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        height: 40
                        radius: Theme.inputRadius
                        color: Theme.infoBg

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingSm

                            Label {
                                text: Icons.info
                                font.family: Icons.family
                                font.pixelSize: 14
                                color: Theme.info
                            }

                            Label {
                                text: "Weather uses your system location services"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }
```

**Step 2: Commit**

```bash
git add crates/myme-ui/qml/pages/SettingsPage.qml
git commit -m "feat(ui): add weather settings section"
```

---

## Task 14: Final Build and Test

**Step 1: Build Rust crates**

Run: `cargo build --release`
Expected: All crates compile successfully

**Step 2: Build Qt application**

Run from project root:
```bash
cd build-qt && cmake --build . --config Release
```
Expected: Application builds successfully

**Step 3: Run application**

Run: `./build/Release/myme-qt.exe`
Expected: Application launches with weather UI visible

**Step 4: Manual verification checklist**

- [ ] Weather icon visible in sidebar footer
- [ ] Click sidebar weather → WeatherPage opens
- [ ] Click Weather nav item → WeatherPage opens
- [ ] Weather settings section appears in Settings
- [ ] Theme toggle still works

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat(weather): complete weather feature integration"
```

---

## Summary

**Total Tasks:** 14
**Estimated Commits:** ~15

**Files Created:**
- `crates/myme-weather/Cargo.toml`
- `crates/myme-weather/src/lib.rs`
- `crates/myme-weather/src/types.rs`
- `crates/myme-weather/src/cache.rs`
- `crates/myme-weather/src/provider.rs`
- `crates/myme-weather/src/location.rs`
- `crates/myme-ui/src/models/weather_model.rs`
- `crates/myme-ui/qml/components/WeatherCompact.qml`
- `crates/myme-ui/qml/components/WeatherWidget.qml`
- `crates/myme-ui/qml/pages/WeatherPage.qml`

**Files Modified:**
- `Cargo.toml` (root workspace)
- `crates/myme-ui/Cargo.toml`
- `crates/myme-ui/src/models/mod.rs`
- `crates/myme-core/src/config.rs`
- `crates/myme-ui/qml/Icons.qml`
- `crates/myme-ui/qml/Main.qml`
- `crates/myme-ui/qml/pages/SettingsPage.qml`
- `qml.qrc`
