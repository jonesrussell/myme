# Weather Location Fallback Chain

**Date:** 2026-03-03
**Status:** Approved

## Problem

GeoClue2 is unavailable on WSL2 (and potentially other Linux environments without desktop location services). When OS-level geolocation fails, the entire weather feature breaks with no fallback.

## Design

### Fallback Chain: Config > OS > IP

`get_current_location()` tries three location sources in order:

1. **Config** — explicit lat/lon/city in `config.toml` (always wins if present)
2. **OS** — GeoClue2 on Linux, WinRT on Windows (current behavior)
3. **IP** — ip-api.com as last resort

### Config Changes

Add optional location fields to `WeatherConfig` in `myme-core/src/config.rs`:

```rust
pub struct WeatherConfig {
    pub temperature_unit: TemperatureUnit,
    pub refresh_minutes: u32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub city_name: Option<String>,
}
```

Usage in `config.toml`:

```toml
[weather]
latitude = 40.7128
longitude = -74.0060
city_name = "New York"
```

### Location Module Changes

`location.rs` signature changes to accept optional config:

```
get_current_location(config: Option<&WeatherConfig>) -> Result<Location, LocationError>
```

New IP geolocation function:
- GET `http://ip-api.com/json/?fields=status,lat,lon,city`
- 5-second timeout
- Uses existing `reqwest` dependency

### Logging

- INFO on success: "Location from config" / "Location from OS" / "Location from IP geolocation"
- WARN on each fallback step (e.g., "OS location failed: ..., trying IP geolocation")

### Error Handling

If all three methods fail, returns `LocationError::ServiceUnavailable` listing what was tried.

### Threading

`weather_service.rs` passes `WeatherConfig` through to `get_current_location()`. Config is already accessible via `AppServices`.

### Testing

- Config-based location: unit test (no network needed)
- IP geolocation: wiremock integration test
- Fallback chain: unit test with mock config
