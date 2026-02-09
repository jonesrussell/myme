//! Reverse geocoding: convert coordinates to human-readable place names.
//! Uses Nominatim (OpenStreetMap) - free, no API key required.

use crate::types::Location;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/reverse";
const REQUEST_TIMEOUT_SECS: u64 = 10;
const USER_AGENT: &str = "MyMe/0.1.0 (https://github.com/myme)";

#[derive(Debug, Deserialize)]
struct NominatimResponse {
    address: Option<NominatimAddress>,
    #[allow(dead_code)]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
    #[serde(rename = "state_district")]
    state_district: Option<String>,
    state: Option<String>,
    county: Option<String>,
    country: Option<String>,
}

/// Reverse geocode coordinates to a human-readable place name (e.g. "Seattle, WA").
/// Returns `None` on failure or timeout; the caller can fall back to coordinates.
pub async fn reverse_geocode(location: &Location) -> Option<String> {
    if location.city_name.is_some() {
        return location.city_name.clone();
    }

    let client = match Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create geocoding client: {}", e);
            return None;
        }
    };

    let url = format!(
        "{}?lat={}&lon={}&format=json&addressdetails=1&layer=address&zoom=10",
        NOMINATIM_URL,
        location.latitude,
        location.longitude
    );

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("Reverse geocode request failed: {}", e);
            return None;
        }
    };

    if !response.status().is_success() {
        tracing::debug!(
            "Reverse geocode returned status {}",
            response.status()
        );
        return None;
    }

    let body: NominatimResponse = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::debug!("Reverse geocode parse error: {}", e);
            return None;
        }
    };

    let addr = body.address?;

    // Capture state/country before the place chain consumes them
    let state = addr.state.clone();
    let country = addr.country.clone();

    // Prefer city > town > village > municipality for the primary place name
    let place = addr
        .city
        .or(addr.town)
        .or(addr.village)
        .or(addr.municipality)
        .or(addr.state_district)
        .or(addr.county)
        .or(addr.state)
        .or(addr.country)?;

    // Add state/country for disambiguation when different from place
    let suffix = state
        .as_ref()
        .filter(|s| !s.is_empty() && s.as_str() != &place)
        .map(String::as_str)
        .or_else(|| {
            country
                .as_ref()
                .filter(|c| !c.is_empty() && c.as_str() != &place)
                .map(String::as_str)
        });

    let result = match suffix {
        Some(s) if !s.is_empty() && s != &place => format!("{}, {}", place, s),
        _ => place,
    };

    tracing::info!("Reverse geocoded to: {}", result);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run with: cargo test -p myme-weather -- --ignored
    async fn test_reverse_geocode_seattle() {
        let loc = Location {
            latitude: 47.6062,
            longitude: -122.3321,
            accuracy_meters: None,
            city_name: None,
        };
        let name = reverse_geocode(&loc).await;
        assert!(name.is_some());
        let n = name.unwrap();
        assert!(n.to_lowercase().contains("seattle"));
    }

    #[tokio::test]
    async fn test_reverse_geocode_preserves_existing_city() {
        let loc = Location {
            latitude: 47.6062,
            longitude: -122.3321,
            accuracy_meters: None,
            city_name: Some("Seattle".to_string()),
        };
        let name = reverse_geocode(&loc).await;
        assert_eq!(name.as_deref(), Some("Seattle"));
    }
}
