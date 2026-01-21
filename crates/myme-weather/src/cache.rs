// Cache implementation - placeholder for Task 3
use crate::types::{WeatherData, WeatherError};
use std::path::PathBuf;

#[derive(Debug)]
pub struct WeatherCache {
    cache_path: PathBuf,
    data: Option<WeatherData>,
}

impl WeatherCache {
    pub fn new(config_dir: &std::path::Path) -> Self {
        let cache_path = config_dir.join("weather_cache.json");
        Self {
            cache_path,
            data: None,
        }
    }
}
