//! Retry utilities for HTTP operations with exponential backoff.
//!
//! This module provides retry logic for transient network failures:
//! - Timeouts
//! - 5xx server errors
//! - Connection resets
//!
//! It does NOT retry:
//! - 4xx client errors (bad requests, not found, etc.)
//! - Authentication failures (401, 403)

use std::future::Future;
use std::time::Duration;

use reqwest::{Response, StatusCode};

/// Default retry configuration
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 100;
pub const DEFAULT_MAX_DELAY_MS: u64 = 5000;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries (doubles each attempt)
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MS),
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with custom settings
    pub fn new(max_retries: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            initial_delay: Duration::from_millis(initial_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
        }
    }

    /// Calculate the delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        // Exponential backoff: initial_delay * 2^attempt
        let factor = 2u64.saturating_pow(attempt);
        let delay_ms = self.initial_delay.as_millis() as u64 * factor;
        let capped = delay_ms.min(self.max_delay.as_millis() as u64);
        Duration::from_millis(capped)
    }
}

/// Error classification for retry decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Should retry the request
    Retry,
    /// Should not retry - permanent failure
    NoRetry,
}

/// Check if a reqwest error is retryable
pub fn is_retryable_error(error: &reqwest::Error) -> RetryDecision {
    // Timeout errors are retryable
    if error.is_timeout() {
        tracing::debug!("Request timed out, will retry");
        return RetryDecision::Retry;
    }

    // Connection errors (reset, refused) are retryable
    if error.is_connect() {
        tracing::debug!("Connection error, will retry");
        return RetryDecision::Retry;
    }

    // Request errors (body issues) are not retryable
    if error.is_request() {
        tracing::debug!("Request error, not retryable");
        return RetryDecision::NoRetry;
    }

    // Status code errors need further inspection
    if let Some(status) = error.status() {
        return is_retryable_status(status);
    }

    // Default: don't retry unknown errors
    RetryDecision::NoRetry
}

/// Check if a status code is retryable
pub fn is_retryable_status(status: StatusCode) -> RetryDecision {
    // 5xx server errors are retryable
    if status.is_server_error() {
        tracing::debug!("Server error ({}), will retry", status);
        return RetryDecision::Retry;
    }

    // 429 Too Many Requests - should retry with backoff
    if status == StatusCode::TOO_MANY_REQUESTS {
        tracing::debug!("Rate limited (429), will retry");
        return RetryDecision::Retry;
    }

    // 408 Request Timeout - retryable
    if status == StatusCode::REQUEST_TIMEOUT {
        tracing::debug!("Request timeout (408), will retry");
        return RetryDecision::Retry;
    }

    // 4xx client errors are NOT retryable (including 401, 403)
    if status.is_client_error() {
        tracing::debug!("Client error ({}), not retryable", status);
        return RetryDecision::NoRetry;
    }

    // Success codes don't need retry
    if status.is_success() {
        return RetryDecision::NoRetry;
    }

    // Default: don't retry
    RetryDecision::NoRetry
}

/// Execute an HTTP request with retry logic.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async closure that performs the HTTP request
///
/// # Returns
/// The successful response or the last error after all retries exhausted
///
/// # Example
/// ```ignore
/// let response = with_retry(
///     RetryConfig::default(),
///     || async { client.get(url).send().await }
/// ).await?;
/// ```
pub async fn with_retry<F, Fut>(config: RetryConfig, operation: F) -> Result<Response, reqwest::Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Response, reqwest::Error>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            let delay = config.delay_for_attempt(attempt - 1);
            tracing::info!(
                "Retry attempt {} of {}, waiting {:?}",
                attempt,
                config.max_retries,
                delay
            );
            tokio::time::sleep(delay).await;
        }

        match operation().await {
            Ok(response) => {
                let status = response.status();

                // Check if the response status is retryable
                if is_retryable_status(status) == RetryDecision::Retry && attempt < config.max_retries {
                    tracing::warn!(
                        "Request returned retryable status {}, attempt {} of {}",
                        status,
                        attempt + 1,
                        config.max_retries + 1
                    );
                    // Convert to error for next iteration
                    continue;
                }

                // Success or non-retryable status
                if attempt > 0 {
                    tracing::info!("Request succeeded after {} retries", attempt);
                }
                return Ok(response);
            }
            Err(e) => {
                let decision = is_retryable_error(&e);

                if decision == RetryDecision::NoRetry {
                    tracing::debug!("Non-retryable error: {}", e);
                    return Err(e);
                }

                tracing::warn!(
                    "Retryable error on attempt {} of {}: {}",
                    attempt + 1,
                    config.max_retries + 1,
                    e
                );
                last_error = Some(e);
            }
        }
    }

    // All retries exhausted
    tracing::error!("All {} retry attempts exhausted", config.max_retries + 1);
    Err(last_error.expect("at least one error should have occurred"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_millis(5000));
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig::new(3, 100, 5000);

        // First retry: 100ms
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        // Second retry: 200ms
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        // Third retry: 400ms
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        // Fourth retry: 800ms
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_delay_capped_at_max() {
        let config = RetryConfig::new(10, 100, 1000);

        // With 100ms initial and max 1000ms, 2^4 * 100 = 1600 > 1000
        assert_eq!(config.delay_for_attempt(4), Duration::from_millis(1000));
        assert_eq!(config.delay_for_attempt(10), Duration::from_millis(1000));
    }

    #[test]
    fn test_retryable_status_codes() {
        // Server errors should retry
        assert_eq!(is_retryable_status(StatusCode::INTERNAL_SERVER_ERROR), RetryDecision::Retry);
        assert_eq!(is_retryable_status(StatusCode::BAD_GATEWAY), RetryDecision::Retry);
        assert_eq!(is_retryable_status(StatusCode::SERVICE_UNAVAILABLE), RetryDecision::Retry);
        assert_eq!(is_retryable_status(StatusCode::GATEWAY_TIMEOUT), RetryDecision::Retry);

        // Rate limiting should retry
        assert_eq!(is_retryable_status(StatusCode::TOO_MANY_REQUESTS), RetryDecision::Retry);

        // Client errors should NOT retry
        assert_eq!(is_retryable_status(StatusCode::BAD_REQUEST), RetryDecision::NoRetry);
        assert_eq!(is_retryable_status(StatusCode::UNAUTHORIZED), RetryDecision::NoRetry);
        assert_eq!(is_retryable_status(StatusCode::FORBIDDEN), RetryDecision::NoRetry);
        assert_eq!(is_retryable_status(StatusCode::NOT_FOUND), RetryDecision::NoRetry);

        // Success should NOT retry
        assert_eq!(is_retryable_status(StatusCode::OK), RetryDecision::NoRetry);
        assert_eq!(is_retryable_status(StatusCode::CREATED), RetryDecision::NoRetry);
    }
}
