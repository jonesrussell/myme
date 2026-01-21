# Location & Weather Feature Design

**Date:** 2026-01-20
**Branch:** `feature/location-weather`
**Status:** Approved

## Overview

Add weather display to MyMe with three UI touchpoints: a compact sidebar footer widget, a full dashboard widget, and a detailed forecast page. Uses system location services for automatic positioning and Open-Meteo API for weather data.

## Requirements

- Quick-glance weather info always visible in sidebar
- Full current-day details on dashboard
- 7-day forecast with hourly breakdowns on dedicated page
- System location services (OS-level) for positioning
- Auto-detect temperature units from system locale
- 15-minute refresh cycle
- Graceful offline behavior with cached data

## Architecture

### New Crate: `myme-weather`

```
crates/myme-weather/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API exports
│   ├── provider.rs      # Open-Meteo API client
│   ├── location.rs      # System location services
│   ├── cache.rs         # Weather data caching
│   └── types.rs         # Weather data structures
```

### Dependencies

- `reqwest` - HTTP client for Open-Meteo API
- `tokio` - Async runtime
- `serde` - JSON parsing
- `chrono` - Date/time handling
- `windows` crate (Windows) - Geolocation API
- `zbus` (Linux) - D-Bus for GeoClue2

### Data Flow

```
System Location API → myme-weather → Cache → cxx-qt bridge → QML
                           ↓
                    Open-Meteo API
```

## Data Types

```rust
pub struct CurrentWeather {
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub condition: WeatherCondition,
    pub description: String,
    pub updated_at: DateTime<Utc>,
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

pub struct HourlyForecast {
    pub time: NaiveTime,
    pub temperature: f64,
    pub condition: WeatherCondition,
    pub precipitation_chance: u8,
}

pub enum WeatherCondition {
    Clear, PartlyCloudy, Cloudy, Fog,
    Drizzle, Rain, HeavyRain,
    Snow, Sleet, Thunderstorm,
}
```

## Location Services

### Platform Abstraction

```rust
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
}

pub enum LocationError {
    PermissionDenied,
    ServiceUnavailable,
    Timeout,
}

pub async fn get_current_location() -> Result<Location, LocationError>;
```

### Platform Implementations

**Windows:**
- `windows` crate with `Windows.Devices.Geolocation` API
- Request permission via `Geolocator::RequestAccessAsync()`
- Handle `GeolocationAccessStatus` (Allowed, Denied, Unspecified)

**Linux:**
- `zbus` to communicate with GeoClue2 D-Bus service
- Standard on most Linux desktops (GNOME, KDE)
- Falls back gracefully if service isn't running

### Permission Flow

1. First launch: Request location permission
2. If denied: Show message in settings to enable manually
3. Cache last known location as fallback

Location refreshed alongside weather (every 15 min) - no continuous tracking.

## Caching Layer

```rust
pub struct WeatherCache {
    current: Option<CurrentWeather>,
    forecast: Vec<DayForecast>,
    location: Option<Location>,
    last_fetch: Option<DateTime<Utc>>,
}

impl WeatherCache {
    pub fn is_stale(&self) -> bool;       // > 15 min old
    pub fn is_expired(&self) -> bool;     // > 2 hours old
    pub fn save(&self) -> Result<()>;
    pub fn load() -> Result<Self>;
}
```

### Storage

- Location: `{config_dir}/myme/weather_cache.json`
- Format: JSON via serde

### Cache Behavior

| Scenario | Action |
|----------|--------|
| Fresh data (< 15 min) | Use cached, no fetch |
| Stale data (15 min - 2 hr) | Show cached with indicator, fetch in background |
| Expired data (> 2 hr) | Show cached with warning, fetch immediately |
| No cache + no network | Show "unavailable" state |

### Stale Indicator

- `is_stale: bool` passed to QML alongside weather data
- UI shows subtle icon (small clock) when stale
- Tooltip: "Weather data from X minutes ago"

## cxx-qt Bridge

### WeatherModel

```rust
#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        type WeatherModel = super::WeatherModelRust;

        #[qproperty]
        fn temperature(self: &WeatherModel) -> f64;
        #[qproperty]
        fn condition_icon(self: &WeatherModel) -> QString;
        #[qproperty]
        fn is_stale(self: &WeatherModel) -> bool;
        #[qproperty]
        fn is_loading(self: &WeatherModel) -> bool;
        #[qproperty]
        fn error_message(self: &WeatherModel) -> QString;

        #[qinvokable]
        fn refresh(self: Pin<&mut WeatherModel>);
        #[qinvokable]
        fn get_daily_forecast(self: &WeatherModel) -> QVariantList;
        #[qinvokable]
        fn get_hourly_forecast(self: &WeatherModel, day_index: i32) -> QVariantList;
    }
}
```

### Async Pattern

- `refresh()` spawns tokio task, emits signal when complete
- 15-minute timer triggers auto-refresh
- Property updates trigger QML binding refreshes

### Temperature Units

- Detect system locale at startup via Qt's `QLocale`
- Store preference in config, pass to Open-Meteo API
- Conversion happens server-side

## QML Components

### File Structure

```
crates/myme-ui/qml/
├── components/
│   ├── WeatherCompact.qml
│   └── WeatherWidget.qml
└── pages/
    └── WeatherPage.qml
```

### WeatherCompact.qml (Sidebar Footer)

```
Expanded:
┌─────────────────┐
│ ☀️  72°  Sunny  │
└─────────────────┘

Collapsed:
┌─────────┐
│ ☀️  72° │
└─────────┘
```

- Receives `expanded: bool` from parent sidebar
- Smooth transition when toggling (animate width, fade text)
- Clickable → navigates to WeatherPage
- Stale: subtle opacity change or small clock icon
- Loading: spinner instead of icon
- Error: show "—" with tooltip

### WeatherWidget.qml (Dashboard)

```
┌──────────────────────────────┐
│ ☀️  72°F  Sunny              │
│ Feels like 75° • H:78° L:65° │
│ Humidity 45% • Wind 8 mph    │
│ ☀️ 6:32 AM  🌙 8:15 PM       │
└──────────────────────────────┘
```

- Full current day info in Kirigami Card
- Clickable → navigates to WeatherPage

### WeatherPage.qml (Detailed)

- Current conditions (expanded widget style)
- 7-day forecast as scrollable list
- Tap any day → expands to show hourly breakdown
- Pull-to-refresh gesture for manual update

## Settings Integration

### UI Section

```
┌─────────────────────────────────────┐
│ Weather                             │
├─────────────────────────────────────┤
│ Temperature Unit      [Auto ▾]      │
│                       • Auto (System)
│                       • Fahrenheit
│                       • Celsius
│                                     │
│ Location              [Detected ✓]  │
│   "Seattle, WA"       [Refresh]     │
│                                     │
│ Location Permission   [Granted ✓]   │
│   (or [Request Access] button)      │
└─────────────────────────────────────┘
```

### Config Storage

```toml
[weather]
temperature_unit = "auto"  # "auto", "fahrenheit", "celsius"
refresh_interval_minutes = 15
```

### Permission States

- **Granted**: Show detected location with refresh button
- **Denied**: Show message + link to system settings
- **Not requested**: Show "Request Access" button

## Error Handling

### Error States by UI Component

| Error | Sidebar | Dashboard Widget | Weather Page |
|-------|---------|------------------|--------------|
| No network | Cached + stale icon | Cached + "Offline" badge | Banner: "Showing cached data" |
| Location denied | Show "—" | "Enable location in settings" | Settings link |
| Location unavailable | Last known location | Same as above | Same as above |
| API error | Cached or "—" | "Weather unavailable" | Retry button |
| First launch, no cache | Loading spinner | Loading state | Loading state |

### Graceful Degradation Priority

1. Fresh data (best)
2. Stale cached data + indicator
3. Expired cached data + warning
4. "Unavailable" state (last resort)

### Startup Sequence

1. Load cache immediately (instant UI)
2. Check location permission
3. If permitted: fetch fresh data in background
4. If denied: show cached or prompt for settings

### Network Resilience

- Timeout: 10 seconds per request
- Retry: Once on failure, then wait for next 15-min cycle
- No aggressive retries

## Testing Strategy

### Unit Tests

- `types.rs`: WMO code → `WeatherCondition` mapping
- `cache.rs`: Staleness logic, save/load round-trip
- `provider.rs`: JSON parsing with mocked Open-Meteo responses

### Integration Tests

- Full fetch cycle with mocked HTTP (`wiremock` crate)
- Cache persistence across restarts
- Locale detection → correct unit selection

### Manual Testing Checklist

- [ ] Fresh install: permission prompt appears
- [ ] Permission denied: graceful fallback UI
- [ ] Network offline: cached data shown with indicator
- [ ] Network restored: auto-refreshes within 15 min
- [ ] Unit toggle: updates immediately
- [ ] Click sidebar → Weather page opens
- [ ] Click dashboard widget → Weather page opens
- [ ] Pull-to-refresh on Weather page

### Platform Testing

- Windows: Test `Geolocation` API permission flow
- Linux: Test with/without GeoClue2 service running

## Files to Create

| File | Purpose |
|------|---------|
| `crates/myme-weather/Cargo.toml` | Crate manifest |
| `crates/myme-weather/src/lib.rs` | Public exports |
| `crates/myme-weather/src/types.rs` | Data structures |
| `crates/myme-weather/src/provider.rs` | Open-Meteo client |
| `crates/myme-weather/src/location.rs` | Location abstraction |
| `crates/myme-weather/src/cache.rs` | Caching layer |
| `crates/myme-ui/src/models/weather_model.rs` | cxx-qt bridge |
| `crates/myme-ui/qml/components/WeatherCompact.qml` | Sidebar widget |
| `crates/myme-ui/qml/components/WeatherWidget.qml` | Dashboard widget |
| `crates/myme-ui/qml/pages/WeatherPage.qml` | Forecast page |

## Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Add `myme-weather` to workspace members |
| `crates/myme-ui/Cargo.toml` | Add `myme-weather` dependency |
| `crates/myme-ui/src/models/mod.rs` | Export `weather_model` |
| `crates/myme-ui/qml/Main.qml` | Add sidebar footer, navigation |
| `crates/myme-ui/qml/pages/SettingsPage.qml` | Add weather settings section |
| `crates/myme-core/src/config.rs` | Add weather config section |
| `qml.qrc` | Add new QML files |
