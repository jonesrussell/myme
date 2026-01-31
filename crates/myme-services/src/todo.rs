use anyhow::{Context, Result};
use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

use crate::retry::{with_retry, RetryConfig, RetryDecision, is_retryable_status};

/// Note (todo item) from the Godo API
/// Matches Godo's Note model structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,  // UUID from Godo
    pub content: String,  // Note content (1-1000 chars)
    pub done: bool,  // Completion status
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Wrapper for list response from Godo API
#[derive(Debug, Clone, Deserialize)]
struct NotesResponse {
    notes: Vec<Todo>,
}

/// Request to create a new note
#[derive(Debug, Clone, Serialize)]
pub struct TodoCreateRequest {
    pub content: String,
}

/// Request to update an existing note (PATCH endpoint)
#[derive(Debug, Clone, Serialize)]
pub struct TodoUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

/// Configuration for creating a TodoClient
#[derive(Debug, Clone, Default)]
pub struct TodoClientConfig {
    /// Base URL for the Godo API
    pub base_url: String,
    /// JWT token for authentication
    pub jwt_token: Option<String>,
    /// Allow invalid/self-signed certificates (debug builds only)
    ///
    /// WARNING: This is a security risk. Only enable for local development
    /// with self-signed certificates.
    pub allow_invalid_certs: bool,
}

/// Client for the Godo API
#[derive(Debug, Clone)]
pub struct TodoClient {
    base_url: Url,
    client: Arc<Client>,
    jwt_token: Option<String>,
    retry_config: RetryConfig,
}

impl TodoClient {
    /// Create a new todo client with the given base URL and optional JWT token
    ///
    /// This creates a client with secure defaults (no invalid cert bypass).
    /// Use `new_with_config` for explicit control over certificate validation.
    pub fn new(base_url: impl AsRef<str>, jwt_token: Option<String>) -> Result<Self> {
        Self::new_with_config(TodoClientConfig {
            base_url: base_url.as_ref().to_string(),
            jwt_token,
            allow_invalid_certs: false, // Safe default
        })
    }

    /// Create a new todo client with explicit configuration
    ///
    /// Use this constructor when you need to configure certificate validation
    /// for development environments with self-signed certificates.
    pub fn new_with_config(config: TodoClientConfig) -> Result<Self> {
        let base_url = Url::parse(&config.base_url).context("Invalid base URL")?;

        #[allow(unused_mut)]
        let mut builder = Client::builder().timeout(std::time::Duration::from_secs(30));

        // Only allow invalid certs in debug builds AND when explicitly configured
        #[cfg(debug_assertions)]
        if config.allow_invalid_certs {
            tracing::warn!(
                "Accepting invalid certificates for {} - DEVELOPMENT ONLY!",
                base_url
            );
            builder = builder.danger_accept_invalid_certs(true);
        }

        // In release builds, the allow_invalid_certs flag is ignored for safety
        #[cfg(not(debug_assertions))]
        if config.allow_invalid_certs {
            tracing::warn!(
                "allow_invalid_certs is ignored in release builds for security"
            );
        }

        let client = builder.build().context("Failed to build HTTP client")?;

        Ok(Self {
            base_url,
            client: Arc::new(client),
            jwt_token: config.jwt_token,
            retry_config: RetryConfig::default(),
        })
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Update the JWT token for authentication
    pub fn set_jwt_token(&mut self, token: String) {
        self.jwt_token = Some(token);
    }

    /// Build request with authorization header if JWT token is available
    fn build_request(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.jwt_token {
            req.header(header::AUTHORIZATION, format!("Bearer {}", token))
        } else {
            req
        }
    }

    /// Send a request with retry logic for transient failures.
    ///
    /// This wraps the request with exponential backoff retry for:
    /// - Timeout errors
    /// - 5xx server errors
    /// - 429 rate limit errors
    /// - Connection resets
    ///
    /// It does NOT retry 4xx client errors (bad requests, auth failures, etc.)
    async fn send_with_retry<F>(&self, build_request: F) -> Result<Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let response = with_retry(self.retry_config.clone(), || async {
            build_request().send().await
        }).await.context("Failed to send request after retries")?;

        let status = response.status();

        // Check for non-retryable error status codes (4xx except rate limit)
        if !status.is_success() && is_retryable_status(status) == RetryDecision::NoRetry {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        Ok(response)
    }

    /// List all notes/todos
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn list_todos(&self) -> Result<Vec<Todo>> {
        let url = self.base_url.join("/api/v1/notes")
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching notes from: {}", url);

        let response = self.send_with_retry(|| {
            self.build_request(self.client.get(url.clone()))
        }).await?;

        let notes_response = response
            .json::<NotesResponse>()
            .await
            .context("Failed to parse response")?;

        tracing::debug!("Fetched {} notes", notes_response.notes.len());
        Ok(notes_response.notes)
    }

    /// Get a specific note by ID
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn get_todo(&self, id: &str) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching note {} from: {}", id, url);

        let response = self.send_with_retry(|| {
            self.build_request(self.client.get(url.clone()))
        }).await?;

        let todo = response
            .json::<Todo>()
            .await
            .context("Failed to parse response")?;

        Ok(todo)
    }

    /// Create a new note
    #[tracing::instrument(skip(self, request), level = "info")]
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo> {
        let url = self.base_url.join("/api/v1/notes")
            .context("Failed to construct URL")?;

        tracing::debug!("Creating note: {}", request.content);

        // Clone request for potential retries
        let request_json = serde_json::to_value(&request)
            .context("Failed to serialize request")?;

        let response = self.send_with_retry(|| {
            self.build_request(
                self.client.post(url.clone())
                    .json(&request_json)
            )
        }).await?;

        let todo = response
            .json::<Todo>()
            .await
            .context("Failed to parse response")?;

        tracing::info!("Created note with ID: {}", todo.id);
        Ok(todo)
    }

    /// Update an existing note (PATCH request for partial updates)
    #[tracing::instrument(skip(self, request), level = "info")]
    pub async fn update_todo(&self, id: &str, request: TodoUpdateRequest) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Updating note {}", id);

        // Clone request for potential retries
        let request_json = serde_json::to_value(&request)
            .context("Failed to serialize request")?;

        let response = self.send_with_retry(|| {
            self.build_request(
                self.client.patch(url.clone())
                    .json(&request_json)
            )
        }).await?;

        let todo = response
            .json::<Todo>()
            .await
            .context("Failed to parse response")?;

        tracing::info!("Updated note {}", id);
        Ok(todo)
    }

    /// Mark a note as done
    pub async fn mark_done(&self, id: &str) -> Result<Todo> {
        self.update_todo(id, TodoUpdateRequest {
            content: None,
            done: Some(true),
        }).await
    }

    /// Mark a note as not done
    pub async fn mark_undone(&self, id: &str) -> Result<Todo> {
        self.update_todo(id, TodoUpdateRequest {
            content: None,
            done: Some(false),
        }).await
    }

    /// Toggle the done status of a note
    pub async fn toggle_done(&self, id: &str) -> Result<Todo> {
        // First get the current note to know its done status
        let note = self.get_todo(id).await?;
        self.update_todo(id, TodoUpdateRequest {
            content: None,
            done: Some(!note.done),
        }).await
    }

    /// Delete a note
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn delete_todo(&self, id: &str) -> Result<()> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Deleting note {}", id);

        let _response = self.send_with_retry(|| {
            self.build_request(self.client.delete(url.clone()))
        }).await?;

        tracing::info!("Deleted note {}", id);
        Ok(())
    }

    /// Check API health (no authentication required)
    ///
    /// Note: Health checks use retry logic to handle transient network issues.
    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.join("/api/v1/health")
            .context("Failed to construct URL")?;

        tracing::debug!("Checking API health: {}", url);

        let response = with_retry(self.retry_config.clone(), || async {
            self.client.get(url.clone()).send().await
        }).await.context("Failed to send health check request")?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_serialization() {
        let todo = Todo {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            content: "Test note".to_string(),
            done: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&todo).unwrap();
        assert!(json.contains("Test note"));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn test_create_request_serialization() {
        let req = TodoCreateRequest {
            content: "New note".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"content":"New note"}"#);
    }

    #[test]
    fn test_update_request_partial() {
        let req = TodoUpdateRequest {
            content: None,
            done: Some(true),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"done":true}"#);
    }
}
