# Weather Location Fallback Chain Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Config > OS > IP geolocation fallback chain so weather works in environments without desktop location services (e.g., WSL2).

**Architecture:** Add optional location fields to `WeatherConfig`, a `LocationOverride` type in `myme-weather`, and an IP geolocation function using ip-api.com. The `get_current_location()` function gains an optional override parameter and tries Config > OS > IP in order, logging which method succeeded.

**Tech Stack:** Rust, reqwest (already a dependency), serde, wiremock (tests)

---

### Task 1: Add `LocationOverride` type to myme-weather

**Files:**
- Modify: `crates/myme-weather/src/types.rs:84-91`

**Step 1: Write the failing test**

Add to the existing `mod tests` in `crates/myme-weather/src/types.rs`:

```rust
#[test]
fn test_location_override_to_location() {
    let ovr = LocationOverride {
        latitude: 40.7128,
        longitude: -74.0060,
        city_name: Some("New York".to_string()),
    };
    let loc: Location = ovr.into();
    assert!((loc.latitude - 40.7128).abs() < f64::EPSILON);
    assert!((loc.longitude - -74.0060).abs() < f64::EPSILON);
    assert_eq!(loc.city_name.as_deref(), Some("New York"));
    assert!(loc.accuracy_meters.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p myme-weather test_location_override_to_location`
Expected: FAIL — `LocationOverride` doesn't exist

**Step 3: Write minimal implementation**

Add above the `Location` struct in `crates/myme-weather/src/types.rs` (before line 84):

```rust
/// Manual location override from user config
#[derive(Debug, Clone)]
pub struct LocationOverride {
    pub latitude: f64,
    pub longitude: f64,
    pub city_name: Option<String>,
}

impl From<LocationOverride> for Location {
    fn from(o: LocationOverride) -> Self {
        Self {
            latitude: o.latitude,
            longitude: o.longitude,
            accuracy_meters: None,
            city_name: o.city_name,
        }
    }
}
```

Also add to `crates/myme-weather/src/lib.rs`:

```rust
pub use types::LocationOverride;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p myme-weather test_location_override_to_location`
Expected: PASS

**Step 5: Commit**

```
feat(weather): add LocationOverride type for manual location config
```

---

### Task 2: Add IP geolocation function

**Files:**
- Modify: `crates/myme-weather/src/location.rs`

**Step 1: Write the failing test**

Add a wiremock integration test in `crates/myme-weather/src/location.rs` inside `mod tests`:

```rust
#[tokio::test]
async fn test_ip_geolocation_success() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/json/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "success",
            "lat": 47.6062,
            "lon": -122.3321,
            "city": "Seattle"
        })))
        .mount(&mock_server)
        .await;

    let result = ip_geolocation_with_url(&mock_server.uri()).await;
    assert!(result.is_ok());
    let loc = result.unwrap();
    assert!((loc.latitude - 47.6062).abs() < 0.001);
    assert!((loc.longitude - -122.3321).abs() < 0.001);
    assert_eq!(loc.city_name.as_deref(), Some("Seattle"));
}

#[tokio::test]
async fn test_ip_geolocation_failure() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/json/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "fail",
            "message": "reserved range"
        })))
        .mount(&mock_server)
        .await;

    let result = ip_geolocation_with_url(&mock_server.uri()).await;
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p myme-weather test_ip_geolocation`
Expected: FAIL — `ip_geolocation_with_url` doesn't exist

**Step 3: Write minimal implementation**

Add to `crates/myme-weather/src/location.rs` (after the `is_available` function, before the platform `mod`s):

```rust
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const IP_API_URL: &str = "http://ip-api.com/json/";
const IP_API_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
}

/// IP-based geolocation via ip-api.com
pub async fn ip_geolocation() -> Result<Location, LocationError> {
    ip_geolocation_with_url(IP_API_URL).await
}

/// Testable version that accepts a custom base URL
async fn ip_geolocation_with_url(base_url: &str) -> Result<Location, LocationError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(IP_API_TIMEOUT_SECS))
        .build()
        .map_err(|e| LocationError::Other(e.to_string()))?;

    let url = format!("{}json/?fields=status,lat,lon,city", base_url.trim_end_matches('/'));

    let resp: IpApiResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| LocationError::Other(format!("IP geolocation request failed: {e}")))?
        .json()
        .await
        .map_err(|e| LocationError::Other(format!("IP geolocation parse failed: {e}")))?;

    if resp.status != "success" {
        return Err(LocationError::Other(format!(
            "IP geolocation returned status: {}",
            resp.status
        )));
    }

    let latitude = resp.lat.ok_or(LocationError::Other("Missing latitude".to_string()))?;
    let longitude = resp.lon.ok_or(LocationError::Other("Missing longitude".to_string()))?;

    Ok(Location {
        latitude,
        longitude,
        accuracy_meters: None,
        city_name: resp.city,
    })
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p myme-weather test_ip_geolocation`
Expected: PASS (both tests)

**Step 5: Commit**

```
feat(weather): add IP-based geolocation via ip-api.com
```

---

### Task 3: Add fallback chain to `get_current_location`

**Files:**
- Modify: `crates/myme-weather/src/location.rs:1-19`

**Step 1: Write the failing test**

Add to `mod tests` in `crates/myme-weather/src/location.rs`:

```rust
#[tokio::test]
async fn test_config_override_skips_os_and_ip() {
    let ovr = LocationOverride {
        latitude: 51.5074,
        longitude: -0.1278,
        city_name: Some("London".to_string()),
    };
    let result = get_current_location(Some(&ovr)).await;
    assert!(result.is_ok());
    let loc = result.unwrap();
    assert!((loc.latitude - 51.5074).abs() < f64::EPSILON);
    assert_eq!(loc.city_name.as_deref(), Some("London"));
}

#[tokio::test]
async fn test_no_override_attempts_os() {
    // Without override, function should attempt OS (will fail in CI/WSL2 but shouldn't panic)
    let result = get_current_location(None).await;
    // We don't assert Ok/Err since OS location availability varies
    // Just verify it returns without panicking
    let _ = result;
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p myme-weather test_config_override`
Expected: FAIL — `get_current_location` doesn't accept parameters

**Step 3: Update the function signature and implement fallback**

Replace the `get_current_location` function in `crates/myme-weather/src/location.rs`:

```rust
use crate::types::{Location, LocationError, LocationOverride};

/// Get the current location using fallback chain: Config > OS > IP
pub async fn get_current_location(
    config_override: Option<&LocationOverride>,
) -> Result<Location, LocationError> {
    // 1. Config override (always wins)
    if let Some(ovr) = config_override {
        tracing::info!(
            "Location from config: {}, {} ({})",
            ovr.latitude,
            ovr.longitude,
            ovr.city_name.as_deref().unwrap_or("no city")
        );
        return Ok(ovr.clone().into());
    }

    // 2. OS-level location (GeoClue2 on Linux, WinRT on Windows)
    match os_location().await {
        Ok(loc) => {
            tracing::info!("Location from OS: {}, {}", loc.latitude, loc.longitude);
            return Ok(loc);
        }
        Err(e) => {
            tracing::warn!("OS location failed: {}, trying IP geolocation", e);
        }
    }

    // 3. IP-based geolocation (last resort)
    match ip_geolocation().await {
        Ok(loc) => {
            tracing::info!(
                "Location from IP geolocation: {}, {} ({})",
                loc.latitude,
                loc.longitude,
                loc.city_name.as_deref().unwrap_or("no city")
            );
            Ok(loc)
        }
        Err(e) => {
            tracing::error!("All location methods failed. Last error: {}", e);
            Err(LocationError::ServiceUnavailable)
        }
    }
}

/// OS-level location (platform-specific)
async fn os_location() -> Result<Location, LocationError> {
    #[cfg(target_os = "windows")]
    {
        windows_impl::get_location().await
    }

    #[cfg(target_os = "linux")]
    {
        linux_impl::get_location().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err(LocationError::ServiceUnavailable)
    }
}
```

Also update `is_available` to stay as-is (it doesn't need the fallback chain).

**Step 4: Run tests to verify they pass**

Run: `cargo test -p myme-weather test_config_override`
Expected: PASS (both tests)

Also run: `cargo test -p myme-weather`
Expected: All existing tests still pass

**Step 5: Commit**

```
feat(weather): implement Config > OS > IP location fallback chain
```

---

### Task 4: Add location config fields to `WeatherConfig`

**Files:**
- Modify: `crates/myme-core/src/config.rs:117-130`

**Step 1: Write the failing test**

Add to existing config tests in `crates/myme-core/src/config.rs`:

```rust
#[test]
fn test_weather_config_with_location() {
    let toml = r#"
[weather]
temperature_unit = "celsius"
refresh_minutes = 30
latitude = 40.7128
longitude = -74.0060
city_name = "New York"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.weather.latitude, Some(40.7128));
    assert_eq!(config.weather.longitude, Some(-74.0060));
    assert_eq!(config.weather.city_name.as_deref(), Some("New York"));
}

#[test]
fn test_weather_config_without_location() {
    let toml = r#"
[weather]
temperature_unit = "celsius"
refresh_minutes = 30
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.weather.latitude.is_none());
    assert!(config.weather.longitude.is_none());
    assert!(config.weather.city_name.is_none());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p myme-core test_weather_config_with_location`
Expected: FAIL — `latitude` field doesn't exist on `WeatherConfig`

**Step 3: Add the fields**

Modify `WeatherConfig` in `crates/myme-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Temperature unit preference
    pub temperature_unit: TemperatureUnit,

    /// Refresh interval in minutes
    pub refresh_minutes: u32,

    /// Manual location latitude (overrides OS geolocation)
    pub latitude: Option<f64>,

    /// Manual location longitude (overrides OS geolocation)
    pub longitude: Option<f64>,

    /// Manual city name (shown in weather display)
    pub city_name: Option<String>,
}
```

Update the `Default` impl:

```rust
impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            temperature_unit: TemperatureUnit::Auto,
            refresh_minutes: 15,
            latitude: None,
            longitude: None,
            city_name: None,
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p myme-core test_weather_config`
Expected: PASS

Also run: `cargo test -p myme-core`
Expected: All tests pass

**Step 5: Commit**

```
feat(core): add optional location fields to WeatherConfig
```

---

### Task 5: Thread config through weather service to location

**Files:**
- Modify: `crates/myme-ui/src/services/weather_service.rs:39-79`
- Modify: `crates/myme-ui/src/services/mod.rs:45-47`

**Step 1: Update `request_fetch` to accept and pass config**

In `crates/myme-ui/src/services/weather_service.rs`, change the signature and body:

```rust
use myme_weather::LocationOverride;

pub fn request_fetch(
    tx: &std::sync::mpsc::Sender<WeatherServiceMessage>,
    provider: Arc<WeatherProvider>,
    location_override: Option<LocationOverride>,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(WeatherServiceMessage::FetchDone(Err(WeatherError::NotInitialized)));
            return;
        }
    };

    runtime.spawn(async move {
        // First get location (with fallback chain)
        let mut location = match myme_weather::location::get_current_location(
            location_override.as_ref(),
        )
        .await
        {
            Ok(loc) => {
                tracing::info!("Got location: {}, {}", loc.latitude, loc.longitude);
                loc
            }
            Err(e) => {
                let _ = tx.send(WeatherServiceMessage::FetchDone(Err(WeatherError::Location(
                    e.to_string(),
                ))));
                return;
            }
        };

        // Reverse geocode to get city name (coordinates only from system APIs)
        if location.city_name.is_none() {
            if let Some(name) = myme_weather::reverse_geocode(&location).await {
                location.city_name = Some(name);
            }
        }

        // Then fetch weather
        let result =
            provider.fetch(&location).await.map_err(|e| WeatherError::Network(e.to_string()));
        let _ = tx.send(WeatherServiceMessage::FetchDone(result));
    });
}
```

**Step 2: Update the caller in `weather_model.rs`**

In `crates/myme-ui/src/models/weather_model.rs` around line 262, update the call:

```rust
// Build location override from config
let config = myme_core::Config::load_cached();
let location_override = match (config.weather.latitude, config.weather.longitude) {
    (Some(lat), Some(lon)) => Some(myme_weather::LocationOverride {
        latitude: lat,
        longitude: lon,
        city_name: config.weather.city_name.clone(),
    }),
    _ => None,
};

request_weather_fetch(&tx, provider, location_override);
```

**Step 3: Update re-export in `services/mod.rs`**

No change needed — the re-export `request_fetch as request_weather_fetch` still works since the function name didn't change.

**Step 4: Verify it compiles**

Run: `cargo test -p myme-core -p myme-weather -p myme-integrations -p myme-services -p myme-auth -p myme-gmail -p myme-calendar`
Expected: All tests pass

Run: `cargo build -p myme-ui` (or `task os:build:rust`)
Expected: Compiles successfully

**Step 5: Commit**

```
feat(weather): thread location config through weather service
```

---

### Task 6: End-to-end verification

**Step 1: Test with config override**

Add to `~/.config/myme/config.toml`:

```toml
[weather]
latitude = 40.7128
longitude = -74.0060
city_name = "New York"
```

**Step 2: Build and run**

Run: `task build && task run`

Expected log output:
```
INFO myme_ui::...: Location from config: 40.7128, -74.006 (New York)
```

Weather should display for New York.

**Step 3: Test without config (IP fallback)**

Remove the lat/lon/city_name lines from config.toml, rebuild and run.

Expected log output:
```
WARN myme_weather::location: OS location failed: ..., trying IP geolocation
INFO myme_weather::location: Location from IP geolocation: ...
```

Weather should display for your approximate IP-based location.

**Step 4: Commit (if any tweaks needed)**

```
fix(weather): tweak location fallback based on e2e testing
```
