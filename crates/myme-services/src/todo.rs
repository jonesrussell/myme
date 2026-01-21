use anyhow::{Context, Result};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

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

/// Client for the Godo API
#[derive(Debug, Clone)]
pub struct TodoClient {
    base_url: Url,
    client: Arc<Client>,
    jwt_token: Option<String>,
}

impl TodoClient {
    /// Create a new todo client with the given base URL and optional JWT token
    pub fn new(base_url: impl AsRef<str>, jwt_token: Option<String>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .context("Invalid base URL")?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(true)  // For development with self-signed certs
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            base_url,
            client: Arc::new(client),
            jwt_token,
        })
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

    /// List all notes/todos
    pub async fn list_todos(&self) -> Result<Vec<Todo>> {
        let url = self.base_url.join("/api/v1/notes")
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching notes from: {}", url);

        let response = self.build_request(self.client.get(url))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let notes_response = response
            .json::<NotesResponse>()
            .await
            .context("Failed to parse response")?;

        tracing::debug!("Fetched {} notes", notes_response.notes.len());
        Ok(notes_response.notes)
    }

    /// Get a specific note by ID
    pub async fn get_todo(&self, id: &str) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching note {} from: {}", id, url);

        let response = self.build_request(self.client.get(url))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let todo = response
            .json::<Todo>()
            .await
            .context("Failed to parse response")?;

        Ok(todo)
    }

    /// Create a new note
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo> {
        let url = self.base_url.join("/api/v1/notes")
            .context("Failed to construct URL")?;

        tracing::debug!("Creating note: {}", request.content);

        let response = self.build_request(
            self.client.post(url)
                .json(&request)
        )
        .send()
        .await
        .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let todo = response
            .json::<Todo>()
            .await
            .context("Failed to parse response")?;

        tracing::info!("Created note with ID: {}", todo.id);
        Ok(todo)
    }

    /// Update an existing note (PATCH request for partial updates)
    pub async fn update_todo(&self, id: &str, request: TodoUpdateRequest) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Updating note {}", id);

        let response = self.build_request(
            self.client.patch(url)
                .json(&request)
        )
        .send()
        .await
        .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

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
    pub async fn delete_todo(&self, id: &str) -> Result<()> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Deleting note {}", id);

        let response = self.build_request(self.client.delete(url))
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        tracing::info!("Deleted note {}", id);
        Ok(())
    }

    /// Check API health (no authentication required)
    pub async fn health_check(&self) -> Result<bool> {
        let url = self.base_url.join("/api/v1/health")
            .context("Failed to construct URL")?;

        tracing::debug!("Checking API health: {}", url);

        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to send health check request")?;

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
