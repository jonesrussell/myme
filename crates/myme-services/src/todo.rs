use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

/// Todo item from the Golang API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Option<u64>,
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Status of a todo item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

/// Request to create a new todo
#[derive(Debug, Clone, Serialize)]
pub struct TodoCreateRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Request to update an existing todo
#[derive(Debug, Clone, Serialize)]
pub struct TodoUpdateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
}

/// Client for the Golang todo API
#[derive(Debug, Clone)]
pub struct TodoClient {
    base_url: Url,
    client: Arc<Client>,
}

impl TodoClient {
    /// Create a new todo client with the given base URL
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .context("Invalid base URL")?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            base_url,
            client: Arc::new(client),
        })
    }

    /// List all todos
    pub async fn list_todos(&self) -> Result<Vec<Todo>> {
        let url = self.base_url.join("/api/todos")
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching todos from: {}", url);

        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        let todos = response
            .json::<Vec<Todo>>()
            .await
            .context("Failed to parse response")?;

        tracing::debug!("Fetched {} todos", todos.len());
        Ok(todos)
    }

    /// Get a specific todo by ID
    pub async fn get_todo(&self, id: u64) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/todos/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Fetching todo {} from: {}", id, url);

        let response = self.client
            .get(url)
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

    /// Create a new todo
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo> {
        let url = self.base_url.join("/api/todos")
            .context("Failed to construct URL")?;

        tracing::debug!("Creating todo: {}", request.title);

        let response = self.client
            .post(url)
            .json(&request)
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

        tracing::info!("Created todo with ID: {:?}", todo.id);
        Ok(todo)
    }

    /// Update an existing todo
    pub async fn update_todo(&self, id: u64, request: TodoUpdateRequest) -> Result<Todo> {
        let url = self.base_url.join(&format!("/api/todos/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Updating todo {}", id);

        let response = self.client
            .put(url)
            .json(&request)
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

        tracing::info!("Updated todo {}", id);
        Ok(todo)
    }

    /// Delete a todo
    pub async fn delete_todo(&self, id: u64) -> Result<()> {
        let url = self.base_url.join(&format!("/api/todos/{}", id))
            .context("Failed to construct URL")?;

        tracing::debug!("Deleting todo {}", id);

        let response = self.client
            .delete(url)
            .send()
            .await
            .context("Failed to send request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        tracing::info!("Deleted todo {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_status_serialization() {
        assert_eq!(
            serde_json::to_string(&TodoStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&TodoStatus::InProgress).unwrap(),
            "\"inprogress\""
        );
        assert_eq!(
            serde_json::to_string(&TodoStatus::Completed).unwrap(),
            "\"completed\""
        );
    }
}
