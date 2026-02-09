# Add a new OAuth provider to MyMe

Use this checklist when adding a new OAuth provider (e.g. **{{providerName}}**).

## Steps

1. **Add the provider in myme-auth**  
   - Create a new file under `crates/myme-auth/src/` (e.g. `{{providerSnake}}.rs`).  
   - Follow the patterns in `github.rs` and `google.rs`: implement the OAuth flow (auth URL, token exchange), and use `SecureStorage` for storing tokens (system keyring: Windows Credential Manager, macOS Keychain, Linux Secret Service).

2. **Wire tokens into service clients**  
   - Where the new provider’s API is used (e.g. a client in myme-services or a dedicated crate), obtain the token from `SecureStorage` and pass it (e.g. as Bearer) to the API client.

3. **Add auth flow in QML (if needed)**  
   - If the user must sign in from the UI, add or reuse an auth model and a settings/auth page that triggers the provider’s auth flow and stores the token via the Rust bridge.

## Conventions

- **Storage**: Always use `SecureStorage` in myme-auth; do not store tokens in plaintext.  
- **Naming**: Use **{{providerName}}** for user-facing names and file/type names (e.g. GitHub → `github.rs`, `GitHubOAuth2Provider`).

Reference: CLAUDE.md "Adding Authentication" and myme-auth `storage.rs`, `github.rs`, `google.rs`.
