# Development Guide

## Project Structure Understanding

### Crate Organization

```
myme (workspace)
├── src/main.rs           → Binary executable (simple demo)
├── crates/
│   ├── myme-core/        → Library: app logic, config, plugins
│   ├── myme-ui/          → Library: Qt/QML bridge
│   └── myme-services/    → Library: API clients
```

**Key Principle**: Libraries do the work, binary is just an entry point.

### Adding New Features

#### 1. Adding a New Service Client

Example: Adding a GitHub API client

```bash
# Edit crates/myme-services/Cargo.toml
# Add: octocrab = "0.x"

# Create crates/myme-services/src/github.rs
```

```rust
use octocrab::Octocrab;

pub struct GitHubClient {
    client: Octocrab,
}

impl GitHubClient {
    pub fn new(token: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()?;
        Ok(Self { client })
    }

    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        // Implementation
    }
}
```

```rust
// In crates/myme-services/src/lib.rs
pub mod github;
pub use github::GitHubClient;
```

#### 2. Adding a New QML Model

Example: Adding a RepoModel

```rust
// Create crates/myme-ui/src/models/repo_model.rs

#[cxx_qt::bridge(cxx_file_stem = "repo_model")]
pub mod ffi {
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        type RepoModel = super::RepoModelRust;

        #[qinvokable]
        fn fetch_repos(self: Pin<&mut RepoModel>);
    }
}

pub struct RepoModelRust {
    loading: bool,
    repos: Vec<Repository>,
    client: Option<Arc<GitHubClient>>,
}
```

Update `build.rs`:
```rust
cxx_qt_build::CxxQtBuilder::new()
    .qml_module("com.myme", "1.0")
    .file("src/models/todo_model.rs")
    .file("src/models/repo_model.rs")  // Add this
    .build();
```

#### 3. Adding a New QML Page

```qml
// Create crates/myme-ui/qml/pages/ReposPage.qml

import QtQuick
import org.kde.kirigami as Kirigami
import com.myme 1.0

Kirigami.ScrollablePage {
    title: "Repositories"

    RepoModel {
        id: repoModel
    }

    ListView {
        model: repoModel.rowCount()
        delegate: Kirigami.BasicListItem {
            label: repoModel.getName(index)
        }
    }
}
```

Update Main.qml navigation:
```qml
Kirigami.Action {
    text: "Repos"
    icon.name: "folder-git"
    onTriggered: pageStack.push("pages/ReposPage.qml")
}
```

### Coding Standards

#### Rust

```rust
// Use Result<T> for fallible operations
pub fn load_config() -> Result<Config> {
    // Never unwrap() in library code
}

// Use anyhow for application errors
use anyhow::{Context, Result};

pub async fn fetch_data() -> Result<Data> {
    let data = api_call()
        .await
        .context("Failed to fetch data from API")?;
    Ok(data)
}

// Use thiserror for library errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

// Always use tracing, not println!
tracing::info!("Starting operation");
tracing::debug!("Details: {:?}", data);
tracing::error!("Operation failed: {}", err);

// Document public APIs
/// Fetches all todos from the API
///
/// # Errors
///
/// Returns an error if the API is unreachable or returns invalid data
pub async fn list_todos(&self) -> Result<Vec<Todo>> {
    // ...
}
```

#### QML

```qml
// Use meaningful IDs
TodoModel {
    id: todoModel  // Not: model1, tm, etc.
}

// Use Kirigami components for consistency
Kirigami.ScrollablePage { }      // Not: Page { }
Kirigami.BasicListItem { }       // Not: Rectangle { }

// Handle loading and error states
ListView {
    visible: !todoModel.loading && todoModel.rowCount() > 0
}

Controls.BusyIndicator {
    visible: todoModel.loading
}

Kirigami.InlineMessage {
    visible: todoModel.errorMessage.length > 0
    text: todoModel.errorMessage
}

// Use property bindings, not imperative code
color: isSelected ? Kirigami.Theme.highlightColor
                  : Kirigami.Theme.backgroundColor

// Not:
onSelectedChanged: {
    if (selected) {
        color = Kirigami.Theme.highlightColor
    } else {
        color = Kirigami.Theme.backgroundColor
    }
}
```

### Testing Strategy

#### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_status_serialization() {
        let status = TodoStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");
    }

    #[tokio::test]
    async fn test_fetch_todos() {
        let client = TodoClient::new("http://localhost:8080").unwrap();
        let todos = client.list_todos().await.unwrap();
        assert!(!todos.is_empty());
    }
}
```

Run tests:
```bash
cargo test --package myme-services
cargo test --workspace
```

#### Integration Tests

Create `tests/integration_test.rs`:
```rust
use myme_core::App;

#[test]
fn test_app_initialization() {
    let app = App::new().expect("Failed to create app");
    assert_eq!(app.plugins().len(), 0);
}
```

#### Manual Testing Checklist

- [ ] Application launches without crashes
- [ ] All navigation items work
- [ ] Todo CRUD operations succeed
- [ ] Error messages display correctly
- [ ] Loading indicators show during operations
- [ ] Configuration persists between runs
- [ ] Logs are written correctly

### Debugging

#### Rust Debugging

```bash
# Enable all debug logs
$env:RUST_LOG="debug"
cargo run

# Enable specific module logs
$env:RUST_LOG="myme_services=debug"
cargo run

# Use rust-lldb or rust-gdb
rust-gdb target/debug/myme
```

#### Qt/QML Debugging

```bash
# Enable QML debugging
$env:QML_IMPORT_TRACE=1
$env:QT_LOGGING_RULES="*.debug=true"
.\build\Release\myme-qt.exe

# QML debugger
qmldebugger
```

#### Common Issues

**cxx-qt bridge not regenerating**:
```bash
cargo clean
cargo build
```

**QML changes not reflected**:
- QML is loaded at runtime, just restart the app
- Check file paths in CMakeLists.txt

**Async operations not completing**:
- Ensure tokio runtime is initialized
- Check that handles are properly passed
- Look for dropped futures

### Performance Tips

#### Rust

```rust
// Use Arc for shared ownership
let client = Arc::new(TodoClient::new(url)?);

// Clone Arc, not data
let client_clone = client.clone();

// Use async for I/O, not CPU-bound tasks
tokio::spawn(async move {
    let result = api_call().await;  // Good: I/O
});

tokio::task::spawn_blocking(|| {
    expensive_computation();  // Good: CPU-bound
});

// Avoid cloning large structures
fn process_todos(&self, todos: &[Todo]) {  // Borrow
    // Not: fn process_todos(&self, todos: Vec<Todo>)
}
```

#### QML

```qml
// Use ListView, not Repeater for large lists
ListView {  // Good: only renders visible items
    model: 1000
    delegate: Item { }
}

// Not:
Repeater {  // Bad: creates all 1000 items
    model: 1000
    delegate: Item { }
}

// Cache expensive computations
readonly property string formattedDate: {
    return Qt.formatDate(date, "yyyy-MM-dd")
}
```

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/github-integration

# Make changes, test
cargo test
cargo build

# Commit with meaningful messages
git add .
git commit -m "Add GitHub API client

- Implement repository listing
- Add OAuth2 authentication
- Create RepoModel for QML binding"

# Push and create PR
git push origin feature/github-integration
```

### Adding Dependencies

#### Rust

```toml
# In root Cargo.toml [workspace.dependencies]
octocrab = "0.38"

# In crate's Cargo.toml
octocrab.workspace = true
```

#### Qt (C++)

```cmake
# In CMakeLists.txt
find_package(Qt6 REQUIRED COMPONENTS Network)

target_link_libraries(myme-qt PRIVATE
    Qt6::Network
)
```

### Configuration Management

```rust
// Add new config field
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub services: ServiceConfig,
    pub ui: UiConfig,
    pub github: GitHubConfig,  // New
}

#[derive(Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub default_org: String,
}

// Update default
impl Default for Config {
    fn default() -> Self {
        Self {
            // ...
            github: GitHubConfig {
                token: None,
                default_org: "myorg".to_string(),
            },
        }
    }
}
```

### Documentation

```rust
//! Module documentation
//!
//! This module provides GitHub API integration.

/// Structure documentation
///
/// # Examples
///
/// ```
/// let client = GitHubClient::new(token)?;
/// let repos = client.list_repos().await?;
/// ```
pub struct GitHubClient { }

// Update README.md with new features
// Update ARCHITECTURE_SUMMARY.md with new components
```

### Release Process

```bash
# 1. Update version in Cargo.toml
version = "0.2.0"

# 2. Update CHANGELOG.md
## [0.2.0] - 2026-01-XX
### Added
- GitHub integration
- Repository management

# 3. Build release
cargo build --release
cmake --build build --config Release

# 4. Test release build
.\build\Release\myme-qt.exe

# 5. Tag and push
git tag v0.2.0
git push origin v0.2.0
```

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [tokio Documentation](https://tokio.rs/)
- [cxx-qt Guide](https://kde.github.io/cxx-qt/)
- [Kirigami Documentation](https://api.kde.org/frameworks/kirigami/html/)
- [Qt QML Documentation](https://doc.qt.io/qt-6/qmlapplications.html)

### Best Practices Summary

✅ Use Result<T> for error handling
✅ Use tracing for logging
✅ Document public APIs
✅ Write tests for core functionality
✅ Use Arc for shared ownership
✅ Use async for I/O operations
✅ Use Kirigami components for UI
✅ Handle loading and error states in QML
✅ Keep crates focused and modular
✅ Update documentation when adding features

---

Happy coding! 🦀
