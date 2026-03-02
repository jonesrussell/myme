# Weather Specification

Covers `crates/myme-weather/`.

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-weather/src/lib.rs` | Re-exports: `WeatherCache`, `WeatherProvider`, `reverse_geocode`, all types |
| `crates/myme-weather/src/provider.rs` | `WeatherProvider` - Open-Meteo API client, response parsing |
| `crates/myme-weather/src/location.rs` | Platform-native geolocation: WinRT (Windows), GeoClue2/D-Bus (Linux) |
| `crates/myme-weather/src/cache.rs` | `WeatherCache` - JSON file-based persistent cache with staleness tracking |
| `crates/myme-weather/src/types.rs` | `WeatherData`, `Location`, `WeatherCondition`, `DayForecast`, error types |
| `crates/myme-weather/src/geocode.rs` | `reverse_geocode()` - Nominatim (OpenStreetMap) reverse geocoding |

## Interface Signatures

### myme-weather::provider

```rust
pub struct WeatherProvider { client: Arc<Client>, unit: TemperatureUnit }
impl WeatherProvider {
    pub fn new(unit: TemperatureUnit) -> Result<Self, WeatherError>;  // 30s timeout, "MyMe/0.1.0" user-agent
    pub fn set_unit(&mut self, unit: TemperatureUnit);
    pub async fn fetch(&self, location: &Location) -> Result<WeatherData, WeatherError>;
}
```

### myme-weather::location

```rust
pub async fn get_current_location() -> Result<Location, LocationError>;
pub async fn is_available() -> bool;

// Windows: uses windows::Devices::Geolocation::Geolocator
//   - Requests access via Geolocator::RequestAccessAsync()
//   - Gets position via GetGeopositionAsync()
//   - Returns latitude, longitude, accuracy_meters

// Linux: uses zbus to talk to org.freedesktop.GeoClue2
//   - Creates client via Manager.CreateClient()
//   - Sets DesktopId = "myme"
//   - Starts client, reads Latitude/Longitude from Location object
//   - Stops client after reading

// Other platforms: returns LocationError::ServiceUnavailable
```

### myme-weather::cache

```rust
pub struct WeatherCache { cache_path: PathBuf, data: Option<WeatherData> }
impl WeatherCache {
    pub fn new(config_dir: &Path) -> Self;         // cache_path = config_dir/weather_cache.json
    pub fn load(&mut self) -> Result<(), WeatherError>;
    pub fn save(&self) -> Result<(), WeatherError>;
    pub fn update(&mut self, data: WeatherData);
    pub fn get(&self) -> Option<&WeatherData>;
    pub fn has_data(&self) -> bool;
    pub fn is_stale(&self) -> bool;                // > 15 minutes old
    pub fn is_expired(&self) -> bool;              // > 2 hours old
    pub fn age_minutes(&self) -> Option<i64>;
}
```

### myme-weather::types

```rust
pub enum TemperatureUnit { Auto, Celsius, Fahrenheit }

pub enum WeatherCondition {
    Clear, PartlyCloudy, Cloudy, Fog, Drizzle, Rain, HeavyRain, Snow, Sleet, Thunderstorm,
}
impl WeatherCondition {
    pub fn from_wmo_code(code: i32) -> Self;       // WMO weather interpretation codes
    pub fn description(&self) -> &'static str;     // e.g., "Partly Cloudy"
    pub fn icon_name(&self) -> &'static str;       // e.g., "cloud_sun" (for QML icon mapping)
}

pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub city_name: Option<String>,
}

pub struct CurrentWeather {
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub condition: WeatherCondition,
    pub updated_at: DateTime<Utc>,
}

pub struct HourlyForecast {
    pub time: NaiveTime,
    pub temperature: f64,
    pub condition: WeatherCondition,
    pub precipitation_chance: u8,
}

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

pub struct WeatherData {
    pub current: CurrentWeather,
    pub forecast: Vec<DayForecast>,     // 7-day forecast
    pub location: Location,
    pub fetched_at: DateTime<Utc>,
}

pub enum LocationError {
    PermissionDenied, ServiceUnavailable, Timeout, Other(String),
}

pub enum WeatherError {
    Network(reqwest::Error), Location(LocationError), Parse(String), Cache(String),
}
```

### myme-weather::geocode

```rust
pub async fn reverse_geocode(location: &Location) -> Option<String>;
// - Returns location.city_name if already set (no API call)
// - Uses Nominatim: GET https://nominatim.openstreetmap.org/reverse?lat=...&lon=...&format=json&addressdetails=1
// - User-Agent: "MyMe/0.1.0 (https://github.com/myme)"
// - 10s timeout
// - Returns "City, State" or "City, Country" format
// - Returns None on any failure (caller falls back to coordinates)
```

## Data Flow

### Weather Fetch (full flow)

```
WeatherModel.fetch_weather() [QML invokable]
  -> sends WeatherServiceMessage::Fetch via channel
  -> Background tokio task:
     1. location::get_current_location().await
        - Windows: WinRT Geolocator API
        - Linux: GeoClue2 D-Bus
     2. geocode::reverse_geocode(&location).await
        - Nominatim API -> "Seattle, Washington"
        - Updates location.city_name
     3. WeatherProvider::fetch(&location).await
        - GET https://api.open-meteo.com/v1/forecast?latitude=...&longitude=...
          &current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code
          &daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_probability_max,sunrise,sunset
          &hourly=temperature_2m,weather_code,precipitation_probability
          &temperature_unit={celsius|fahrenheit}&wind_speed_unit=mph&forecast_days=7&timezone=auto
        - Parses ForecastResponse -> WeatherData
     4. WeatherCache::update(data) + save()
  -> Result sent back via channel
  -> QML Timer polls: weatherModel.poll_channel()
```

### WMO Code Mapping

```
Code 0        -> Clear
Code 1-2      -> PartlyCloudy
Code 3        -> Cloudy
Code 45,48    -> Fog
Code 51,53,55 -> Drizzle
Code 56,57    -> Sleet (freezing drizzle)
Code 61,63,80 -> Rain
Code 65,81,82 -> HeavyRain
Code 66,67    -> Sleet (freezing rain)
Code 71,73,75,77,85,86 -> Snow
Code 95,96,99 -> Thunderstorm
Unknown       -> Clear (default)
```

### Cache Lifecycle

```
App start -> WeatherCache::new(config_dir) -> load() from weather_cache.json
  if is_stale() (>15 min) -> trigger API fetch
  if is_expired() (>2 hours) -> show stale data + fetch
  if no data -> show loading state + fetch
After fetch -> update(data) -> save() to disk
```

## Storage / Schema

### Weather Cache File

- Path: `~/.config/myme/weather_cache.json`
- Format: JSON-serialized `WeatherData` struct (via serde)
- Contains: current conditions, 7-day forecast with hourly data, location, fetch timestamp
- No database; simple JSON file read/write

### Staleness Thresholds

| Threshold | Duration | Behavior |
|-----------|----------|----------|
| Fresh | < 15 min | Use cached data, no fetch |
| Stale | 15 min - 2 hours | Use cached data, fetch in background |
| Expired | > 2 hours | Show stale data prominently, fetch |
| No data | N/A | Show loading, fetch immediately |

## Configuration

| Config Key | Type | Default | Notes |
|-----------|------|---------|-------|
| `weather.temperature_unit` | enum | auto | `auto`, `celsius`, `fahrenheit` |
| `weather.refresh_minutes` | u32 | 15 | UI polling interval (separate from cache staleness) |

### API Details

| Service | URL | Auth | Rate Limit |
|---------|-----|------|------------|
| Open-Meteo | `https://api.open-meteo.com/v1/forecast` | None (free) | 10,000/day |
| Nominatim | `https://nominatim.openstreetmap.org/reverse` | None (free) | 1 req/sec (User-Agent required) |

## Edge Cases

- **Location permission denied**: `LocationError::PermissionDenied` on Windows (Geolocator access denied) or Linux (D-Bus access denied)
- **GeoClue2 not installed**: Linux `is_available()` pings D-Bus; returns false if service not running
- **Nominatim failure**: `reverse_geocode()` returns `None`; caller uses raw coordinates as fallback
- **Nominatim rate limit**: User-Agent required by Nominatim terms; 10s timeout prevents blocking
- **Unknown WMO code**: Defaults to `WeatherCondition::Clear` (safe fallback)
- **Cache file missing**: `load()` returns `Ok(())` with `data = None`; `is_stale()` and `is_expired()` both return true
- **Cache directory creation**: `save()` calls `create_dir_all` for parent directory
- **Temperature unit "auto"**: Sent to Open-Meteo as `celsius`; UI could potentially auto-detect locale
- **Humidity clamping**: API humidity value clamped to 0-255 for u8 storage
- **Precipitation clamping**: API probability clamped to 0-100 for u8 storage
- **Sunrise/sunset parsing**: Extracted from datetime strings like "2026-01-20T06:32" by splitting on 'T' and parsing "%H:%M"
- **Hourly data grouping**: Hourly entries filtered to match each day's date prefix for per-day hourly forecast
- **Network timeout**: 30s timeout on weather API requests; returns `WeatherError::Network`
- **Cache serialization**: Uses `serde_json::to_string_pretty` for human-readable cache files
