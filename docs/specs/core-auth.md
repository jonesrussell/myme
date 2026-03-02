# Core & Authentication Specification

Covers `crates/myme-core/` and `crates/myme-auth/`.

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-core/src/lib.rs` | Re-exports, tracing initialization via `init()` |
| `crates/myme-core/src/app.rs` | `App` struct: lifecycle manager holding `Arc<Config>` |
| `crates/myme-core/src/config.rs` | TOML config loading, validation, caching, all config structs |
| `crates/myme-core/src/error.rs` | Typed error hierarchy: `AppError`, `AuthError`, `GitHubError`, etc. |
| `crates/myme-auth/src/lib.rs` | Re-exports `GitHubAuth`, `GoogleOAuth2Provider`, `OAuth2Provider`, `SecureStorage`, `TokenSet` |
| `crates/myme-auth/src/oauth.rs` | `OAuth2Config`, `OAuth2Provider` trait, PKCE flow, warp callback server |
| `crates/myme-auth/src/storage.rs` | `SecureStorage` (keyring), `TokenSet`, legacy migration |
| `crates/myme-auth/src/github.rs` | `GitHubAuth` implementing `OAuth2Provider` |
| `crates/myme-auth/src/google.rs` | `GoogleOAuth2Provider` for Gmail/Calendar OAuth |

## Interface Signatures

### myme-core::config

```rust
// Config structs
pub struct Config {
    pub config_dir: PathBuf,
    pub services: ServiceConfig,
    pub ui: UiConfig,             // window_width: u32, window_height: u32, dark_mode: bool
    pub weather: WeatherConfig,   // temperature_unit: TemperatureUnit, refresh_minutes: u32
    pub projects: ProjectsConfig, // sync_interval_minutes: u32, auto_create_labels: bool
    pub repos: ReposConfig,       // local_search_path: String
    pub github: GitHubConfig,     // client_id: String, client_secret: String
    pub google: Option<GoogleConfig>, // client_id: Option<String>, client_secret: Option<String>
    pub notes: NotesConfig,       // sqlite_path: String
}

pub enum TemperatureUnit { Auto, Celsius, Fahrenheit }

impl Config {
    pub fn load() -> Result<Self>;
    pub fn load_cached() -> Arc<Self>;       // OnceLock-backed, process-lifetime cache
    pub fn load_validated() -> Result<(Self, ValidationResult)>;
    pub fn validate(&self) -> ValidationResult;
    pub fn save(&self) -> Result<()>;
}

impl GitHubConfig {
    pub fn is_configured(&self) -> bool;     // not empty, not "YOUR_" prefix
}

impl GoogleConfig {
    pub fn is_configured(&self) -> bool;
}

impl ReposConfig {
    pub fn effective_local_search_path(&self) -> (PathBuf, bool); // (path, config_invalid)
}

impl NotesConfig {
    pub fn sqlite_path(&self) -> PathBuf;    // expands ~/
}

pub struct ValidationResult {
    pub errors: Vec<ConfigValidationError>,
    pub warnings: Vec<ConfigValidationError>,
}
impl ValidationResult {
    pub fn is_valid(&self) -> bool;
    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>);
    pub fn add_warning(&mut self, field: impl Into<String>, message: impl Into<String>);
    pub fn error_summary(&self) -> String;
}
```

### myme-core::error

```rust
pub enum AppError {
    Network(NetworkError), Database(DatabaseError), Config(ConfigError),
    Auth(AuthError), GitHub(GitHubError), Weather(WeatherError),
    Io(std::io::Error), Service(String), Other(anyhow::Error),
}
impl AppError { pub fn user_message(&self) -> &'static str; }

pub enum AuthError {
    TokenExpired, TokenNotFound(String), InvalidToken,
    OAuthFailed(String), OAuthCancelled, InvalidCredentials,
    StorageError(String), PortInUse(u16),
}

pub enum GitHubError {
    RateLimited { reset_time: String }, RepoNotFound { owner: String, repo: String },
    Unauthorized, Forbidden, ApiError { status: u16, message: String },
    NotAuthenticated, InvalidRepoUrl(String),
}
impl GitHubError { pub fn message(msg: impl Into<String>) -> Self; }

pub trait ReqwestErrorExt { fn into_network_error(self) -> NetworkError; }
pub trait RusqliteErrorExt { fn into_database_error(self) -> DatabaseError; }
```

### myme-core::app

```rust
pub struct App { config: Arc<Config> }
impl App {
    pub fn new() -> Result<Self>;
    pub fn initialize(&mut self) -> Result<()>;
    pub fn shutdown(&mut self) -> Result<()>;
    pub fn config(&self) -> &Config;
}
```

### myme-auth::storage

```rust
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,          // Unix timestamp
    pub scopes: Vec<String>,
}
impl TokenSet {
    pub fn needs_refresh(&self) -> bool;  // within 5 min of expiry
    pub fn is_expired(&self) -> bool;
}

pub struct SecureStorage;
impl SecureStorage {
    pub fn store_token(service: &str, token_set: &TokenSet) -> Result<()>;
    pub fn retrieve_token(service: &str) -> Result<TokenSet>;  // keyring first, then legacy migration
    pub fn delete_token(service: &str) -> Result<()>;
    pub fn has_token(service: &str) -> bool;
}
```

### myme-auth::oauth

```rust
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

pub trait OAuth2Provider: Send + Sync {
    fn service_id(&self) -> &str;
    fn config(&self) -> &OAuth2Config;
    fn authorize(&self) -> Result<(String, CsrfToken, PkceCodeVerifier)>;
    async fn exchange_code(&self, code: String, pkce_verifier: PkceCodeVerifier) -> Result<TokenSet>;
    async fn authenticate(&self) -> Result<TokenSet>;  // full browser flow
    fn get_token(&self) -> Option<TokenSet>;
    fn is_authenticated(&self) -> bool;
    fn sign_out(&self) -> Result<()>;
}
```

### myme-auth::github

```rust
pub struct GitHubAuth { config: OAuth2Config }
impl GitHubAuth {
    pub fn new(client_id: String, client_secret: String) -> Self; // scopes: repo, read:user, user:email
    pub fn with_scopes(client_id: String, client_secret: String, scopes: Vec<String>) -> Self;
}
impl OAuth2Provider for GitHubAuth { fn service_id(&self) -> &str { "github" } }
```

### myme-auth::google

```rust
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
}

pub struct GoogleUserInfo {
    pub email: String,
    pub verified_email: bool,
    pub picture: Option<String>,
}

pub struct GoogleOAuth2Provider { pub client_id: String, pub client_secret: String }
impl GoogleOAuth2Provider {
    pub fn new(client_id: String, client_secret: String) -> Self;
    pub fn authorization_url(&self, port: u16) -> (String, String); // (url, state)
    pub async fn exchange_code(&self, code: &str, port: u16) -> Result<GoogleTokenResponse>;
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleTokenResponse>;
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo>;
}
```

## Data Flow

### OAuth2 Authentication (GitHub)

1. `GitHubAuth::authenticate()` finds available port (8080-8089)
2. Generates PKCE challenge + CSRF token
3. Builds authorization URL with dynamic redirect URI
4. Starts warp callback server on discovered port
5. Opens browser via `webbrowser::open()`; logs URL on failure
6. Waits for callback via `oneshot::channel`
7. Validates CSRF token from callback
8. Exchanges authorization code for token via `exchange_code()`
9. `SecureStorage::store_token("github", &token_set)` saves to system keyring

### Google OAuth2 Authentication

1. `GoogleOAuth2Provider::authorization_url(port)` builds URL with scopes: `gmail.modify`, `calendar`, `userinfo.email`
2. URL includes `access_type=offline` and `prompt=consent` for refresh token
3. `exchange_code(code, port)` POSTs to `https://oauth2.googleapis.com/token`
4. `refresh_token(refresh_token)` renews expired access tokens
5. `get_user_info(access_token)` fetches email from `https://www.googleapis.com/oauth2/v2/userinfo`

### Token Retrieval with Legacy Migration

1. `SecureStorage::retrieve_token(service)` checks system keyring
2. On keyring miss, checks `~/.config/myme/tokens/{service}.json`
3. If legacy file found: stores in keyring, deletes file
4. Returns token or error

## Storage / Schema

### Config File

- Path: `~/.config/myme/config.toml` (Linux), `%APPDATA%\myme\config.toml` (Windows)
- Format: TOML, created with defaults on first run
- Cached via `OnceLock<Arc<Config>>` for hot paths

### Token Storage

- Primary: System keyring (service=`myme`, username=service_id)
  - Windows: Windows Credential Manager
  - macOS: Keychain
  - Linux: Secret Service (libsecret)
- Legacy: `~/.config/myme/tokens/{github,google}.json` (auto-migrated)

## Configuration

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `ui.window_width` | u32 | 1200 | Validated >0, warns >10000 |
| `ui.window_height` | u32 | 800 | Validated >0, warns >10000 |
| `ui.dark_mode` | bool | false | |
| `weather.temperature_unit` | enum | auto | auto/celsius/fahrenheit |
| `weather.refresh_minutes` | u32 | 15 | 0 disables, warns >1440 |
| `projects.sync_interval_minutes` | u32 | 5 | 0 disables |
| `projects.auto_create_labels` | bool | true | |
| `repos.local_search_path` | String | ~/dev | Validated exists+is_dir |
| `github.client_id` | String | YOUR_GITHUB_CLIENT_ID | |
| `github.client_secret` | String | YOUR_GITHUB_CLIENT_SECRET | |
| `google.client_id` | Option<String> | None | |
| `google.client_secret` | Option<String> | None | |
| `notes.sqlite_path` | String | ~/.config/myme/notes.db | ~ expanded |

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `RUST_LOG` | Tracing filter (e.g., `info`, `debug`) |

## Edge Cases

- **Port conflict**: OAuth callback tries ports 8080-8089 sequentially; fails with `AuthError::PortInUse` if all occupied
- **Browser open failure**: Logs URL to console; user can paste manually; flow continues
- **Token expiry buffer**: `TokenSet::needs_refresh()` triggers 5 minutes before actual expiry
- **CSRF mismatch**: `authenticate()` bails with "CSRF token mismatch" if state param differs
- **Keyring unavailable**: Falls back to legacy file; if both fail, returns error
- **Config parse failure**: `load_cached()` falls back to `Config::default()` with tracing warning
- **Validation**: `load_validated()` returns `Err` for critical errors (zero window dims), `Ok` with warnings for non-critical (unconfigured GitHub, missing paths)
- **GitHub tokens don't expire**: Default `expires_in` set to 1 year (365*24*3600 seconds)
- **Google refresh tokens**: Only issued on first consent (`prompt=consent`); subsequent flows may lack refresh_token
