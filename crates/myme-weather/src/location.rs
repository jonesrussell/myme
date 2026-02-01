use crate::types::{Location, LocationError};

/// Get the current location from the system
pub async fn get_current_location() -> Result<Location, LocationError> {
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

        let coord = position
            .Coordinate()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let point = coord
            .Point()
            .map_err(|e| LocationError::Other(e.to_string()))?;

        let pos = point
            .Position()
            .map_err(|e| LocationError::Other(e.to_string()))?;

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

        let location_value: zbus::zvariant::OwnedValue = location_reply
            .body()
            .deserialize()
            .map_err(|e| LocationError::Other(e.to_string()))?;

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

        let lat_value: zbus::zvariant::OwnedValue = lat_reply
            .body()
            .deserialize()
            .map_err(|e| LocationError::Other(e.to_string()))?;

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

        let lon_value: zbus::zvariant::OwnedValue = lon_reply
            .body()
            .deserialize()
            .map_err(|e| LocationError::Other(e.to_string()))?;

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
