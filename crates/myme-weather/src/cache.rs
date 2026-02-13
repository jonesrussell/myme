use crate::types::{WeatherData, WeatherError};
use chrono::{Duration, Utc};
use std::path::PathBuf;

const STALE_THRESHOLD_MINUTES: i64 = 15;
const EXPIRED_THRESHOLD_HOURS: i64 = 2;

/// Persistent weather cache
#[derive(Debug, Clone)]
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

        self.data =
            serde_json::from_str(&contents).map_err(|e| WeatherError::Cache(e.to_string()))?;

        Ok(())
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<(), WeatherError> {
        if let Some(data) = &self.data {
            if let Some(parent) = self.cache_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| WeatherError::Cache(e.to_string()))?;
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
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use crate::types::*;
    use chrono::{DateTime, NaiveDate, Utc};
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
        assert!((10..=11).contains(&age));
    }
}
