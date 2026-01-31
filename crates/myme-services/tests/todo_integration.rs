//! Integration tests for TodoClient using wiremock.
//!
//! These tests verify the TodoClient behavior against a mock HTTP server.

use myme_services::{TodoClient, TodoCreateRequest, TodoUpdateRequest};
use wiremock::matchers::{method, path, header};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a test todo
fn test_todo(id: &str, content: &str, done: bool) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "content": content,
        "done": done,
        "created_at": "2026-01-30T12:00:00Z",
        "updated_at": "2026-01-30T12:00:00Z"
    })
}

#[tokio::test]
async fn test_list_todos_success() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("GET"))
        .and(path("/api/v1/notes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notes": [
                test_todo("1", "First note", false),
                test_todo("2", "Second note", true),
            ]
        })))
        .mount(&mock_server)
        .await;

    // Create client pointing to mock server
    let client = TodoClient::new(&mock_server.uri(), None).unwrap();

    // Test the client
    let todos = client.list_todos().await.unwrap();

    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].id, "1");
    assert_eq!(todos[0].content, "First note");
    assert!(!todos[0].done);
    assert_eq!(todos[1].id, "2");
    assert!(todos[1].done);
}

#[tokio::test]
async fn test_list_todos_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/notes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notes": []
        })))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let todos = client.list_todos().await.unwrap();

    assert!(todos.is_empty());
}

#[tokio::test]
async fn test_get_todo_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/notes/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(test_todo("abc123", "Test note", false)))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let todo = client.get_todo("abc123").await.unwrap();

    assert_eq!(todo.id, "abc123");
    assert_eq!(todo.content, "Test note");
}

#[tokio::test]
async fn test_get_todo_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/notes/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": "Note not found"
        })))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let result = client.get_todo("nonexistent").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("404"), "Error should mention 404 status: {}", err);
}

#[tokio::test]
async fn test_create_todo_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/notes"))
        .respond_with(ResponseTemplate::new(201).set_body_json(test_todo("new-id", "New todo", false)))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let todo = client.create_todo(TodoCreateRequest {
        content: "New todo".to_string(),
    }).await.unwrap();

    assert_eq!(todo.id, "new-id");
    assert_eq!(todo.content, "New todo");
    assert!(!todo.done);
}

#[tokio::test]
async fn test_update_todo_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/api/v1/notes/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(test_todo("abc123", "Updated content", true)))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let todo = client.update_todo("abc123", TodoUpdateRequest {
        content: Some("Updated content".to_string()),
        done: Some(true),
    }).await.unwrap();

    assert_eq!(todo.content, "Updated content");
    assert!(todo.done);
}

#[tokio::test]
async fn test_delete_todo_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/notes/abc123"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let result = client.delete_todo("abc123").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_check_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy"
        })))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let healthy = client.health_check().await.unwrap();

    assert!(healthy);
}

#[tokio::test]
async fn test_health_check_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let healthy = client.health_check().await.unwrap();

    assert!(!healthy);
}

#[tokio::test]
async fn test_retry_on_server_error() {
    let mock_server = MockServer::start().await;

    // First call returns 500, subsequent calls return 200
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let healthy = client.health_check().await.unwrap();

    // Should succeed after retries
    assert!(healthy);
}

#[tokio::test]
async fn test_no_retry_on_client_error() {
    let mock_server = MockServer::start().await;

    // 401 should not retry
    Mock::given(method("GET"))
        .and(path("/api/v1/notes"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Unauthorized"
        })))
        .expect(1) // Should only be called once (no retries)
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), None).unwrap();
    let result = client.list_todos().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_token_included() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/notes"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "notes": []
        })))
        .mount(&mock_server)
        .await;

    let client = TodoClient::new(&mock_server.uri(), Some("test-token".to_string())).unwrap();
    let result = client.list_todos().await;

    // If the header wasn't present, the mock wouldn't match and we'd get an error
    assert!(result.is_ok());
}
