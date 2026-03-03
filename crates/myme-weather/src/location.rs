use crate::types::{Location, LocationError, LocationOverride};

/// Get the current location using a fallback chain: Config > OS > IP geolocation.
pub async fn get_current_location(
    config_override: Option<&LocationOverride>,
) -> Result<Location, LocationError> {
    // 1. Config override — return immediately if provided
    if let Some(ovr) = config_override {
        tracing::info!("Location from config");
        return Ok(Location::from(ovr.clone()));
    }

    // 2. OS-level location service
    match os_location().await {
        Ok(loc) => {
            tracing::info!("Location from OS");
            return Ok(loc);
        }
        Err(err) => {
            tracing::warn!("OS location failed: {err}, trying IP geolocation");
        }
    }

    // 3. IP geolocation as last resort
    match ip_geolocation(None).await {
        Ok(loc) => {
            tracing::info!("Location from IP geolocation");
            Ok(loc)
        }
        Err(ip_err) => {
            tracing::error!("All location methods failed. Last error: {ip_err}");
            Err(LocationError::ServiceUnavailable)
        }
    }
}

/// Get location from the OS-level location service.
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

/// Response from ip-api.com
#[derive(serde::Deserialize)]
struct IpApiResponse {
    status: String,
    lat: Option<f64>,
    lon: Option<f64>,
    city: Option<String>,
}

/// Get approximate location via IP geolocation (ip-api.com).
///
/// Accepts an optional `base_url` for testing with wiremock.
/// When `None`, defaults to `http://ip-api.com`.
pub(crate) async fn ip_geolocation(base_url: Option<&str>) -> Result<Location, LocationError> {
    let base = base_url.unwrap_or("http://ip-api.com");
    let url = format!("{base}/json/?fields=status,lat,lon,city");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| LocationError::Other(format!("IP geolocation request failed: {e}")))?;

    let data: IpApiResponse = resp
        .json()
        .await
        .map_err(|e| LocationError::Other(format!("IP geolocation parse failed: {e}")))?;

    if data.status != "success" {
        return Err(LocationError::Other(format!(
            "IP geolocation returned status: {}",
            data.status
        )));
    }

    let latitude =
        data.lat.ok_or_else(|| LocationError::Other("IP geolocation: missing lat".to_string()))?;
    let longitude =
        data.lon.ok_or_else(|| LocationError::Other("IP geolocation: missing lon".to_string()))?;

    Ok(Location { latitude, longitude, accuracy_meters: None, city_name: data.city })
}

/// Check if location services are available
pub async fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_impl::is_available().await
    }

    #[cfg(target_os = "linux")]
    {
        linux_impl::is_available().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionAccuracy};

    pub async fn is_available() -> bool {
        match Geolocator::RequestAccessAsync() {
            Ok(op) => match op.get() {
                Ok(status) => status == GeolocationAccessStatus::Allowed,
                Err(_) => false,
            },
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
        let geolocator = Geolocator::new().map_err(|e| LocationError::Other(e.to_string()))?;

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

        let coord = position.Coordinate().map_err(|e| LocationError::Other(e.to_string()))?;

        let point = coord.Point().map_err(|e| LocationError::Other(e.to_string()))?;

        let pos = point.Position().map_err(|e| LocationError::Other(e.to_string()))?;

        let accuracy = coord.Accuracy().ok();

        Ok(Location {
            latitude: pos.Latitude,
            longitude: pos.Longitude,
            accuracy_meters: accuracy,
            city_name: None, // Would need reverse geocoding
        })
    }
}

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;
    use zbus::Connection;

    const GEOCLUE_BUS: &str = "org.freedesktop.GeoClue2";
    const GEOCLUE_MANAGER_PATH: &str = "/org/freedesktop/GeoClue2/Manager";

    pub async fn is_available() -> bool {
        match Connection::system().await {
            Ok(conn) => conn
                .call_method(
                    Some(GEOCLUE_BUS),
                    GEOCLUE_MANAGER_PATH,
                    Some("org.freedesktop.DBus.Peer"),
                    "Ping",
                    &(),
                )
                .await
                .is_ok(),
            Err(_) => false,
        }
    }

    pub async fn get_location() -> Result<Location, LocationError> {
        let conn = Connection::system().await.map_err(|_| LocationError::ServiceUnavailable)?;

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

        let client_path: zbus::zvariant::OwnedObjectPath =
            reply.body().deserialize().map_err(|e| LocationError::Other(e.to_string()))?;

        // Set desktop ID (required)
        conn.call_method(
            Some(GEOCLUE_BUS),
            client_path.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Set",
            &("org.freedesktop.GeoClue2.Client", "DesktopId", zbus::zvariant::Value::from("myme")),
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

        let location_value: zbus::zvariant::OwnedValue =
            location_reply.body().deserialize().map_err(|e| LocationError::Other(e.to_string()))?;

        let location_path: zbus::zvariant::OwnedObjectPath = location_value
            .try_into()
            .map_err(|_| LocationError::Other("Invalid location path".to_string()))?;

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

        let lat_value: zbus::zvariant::OwnedValue =
            lat_reply.body().deserialize().map_err(|e| LocationError::Other(e.to_string()))?;

        let latitude: f64 = lat_value
            .try_into()
            .map_err(|_| LocationError::Other("Invalid latitude".to_string()))?;

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

        let lon_value: zbus::zvariant::OwnedValue =
            lon_reply.body().deserialize().map_err(|e| LocationError::Other(e.to_string()))?;

        let longitude: f64 = lon_value
            .try_into()
            .map_err(|_| LocationError::Other("Invalid longitude".to_string()))?;

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

        Ok(Location { latitude, longitude, accuracy_meters: None, city_name: None })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // These tests require actual location services, so they're ignored by default
    #[tokio::test]
    #[ignore]
    async fn test_location_available() {
        let available = is_available().await;
        println!("Location available: {}", available);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_location_from_os() {
        let result = get_current_location(None).await;
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

    #[tokio::test]
    async fn test_config_override_returns_immediately() {
        let ovr = LocationOverride {
            latitude: 48.8566,
            longitude: 2.3522,
            city_name: Some("Paris".to_string()),
        };
        let loc = get_current_location(Some(&ovr)).await.unwrap();
        assert!((loc.latitude - 48.8566).abs() < f64::EPSILON);
        assert!((loc.longitude - 2.3522).abs() < f64::EPSILON);
        assert_eq!(loc.city_name.as_deref(), Some("Paris"));
    }

    #[tokio::test]
    async fn test_ip_geolocation_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/json/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "lat": 37.7749,
                "lon": -122.4194,
                "city": "San Francisco"
            })))
            .mount(&server)
            .await;

        let loc = ip_geolocation(Some(&server.uri())).await.unwrap();
        assert!((loc.latitude - 37.7749).abs() < f64::EPSILON);
        assert!((loc.longitude - (-122.4194)).abs() < f64::EPSILON);
        assert_eq!(loc.city_name.as_deref(), Some("San Francisco"));
    }

    #[tokio::test]
    async fn test_ip_geolocation_error_status() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/json/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "fail",
                "lat": null,
                "lon": null,
                "city": null
            })))
            .mount(&server)
            .await;

        let err = ip_geolocation(Some(&server.uri())).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("fail"), "Error should mention status: {msg}");
    }

    #[tokio::test]
    async fn test_ip_geolocation_server_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/json/.*"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let err = ip_geolocation(Some(&server.uri())).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("parse failed"), "Should fail to parse: {msg}");
    }
}
