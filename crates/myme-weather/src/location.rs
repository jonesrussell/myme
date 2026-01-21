// Location implementation - placeholder for Task 5
use crate::types::{Location, LocationError};

pub async fn get_current_location() -> Result<Location, LocationError> {
    Err(LocationError::ServiceUnavailable)
}

pub async fn is_available() -> bool {
    false
}
