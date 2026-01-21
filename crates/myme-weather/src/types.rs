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
