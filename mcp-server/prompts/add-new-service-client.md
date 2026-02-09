# Add a new service client to MyMe

Use this checklist when adding a new backend client. **{{crateName}}** is optional; if provided, use it for crate/module naming.

## Steps

1. **Add the client module in myme-services**  
   - Create or extend modules under `crates/myme-services/src/`.  
   - Use async methods, `anyhow::Result` for errors, and `tracing` for logging.  
   - Follow existing patterns (e.g. `GitHubClient`, `NoteClient`).

2. **Export from the crate**  
   - In `crates/myme-services/src/lib.rs`, export the new client (e.g. `pub use foo::FooClient;`).

3. **If the feature needs a UI**  
   - Add a QObject model in `crates/myme-ui/src/models/` using the channel pattern (no `block_on()`).  
   - Register the model in `crates/myme-ui/build.rs`.  
   - Optionally register the service and channel in `app_services.rs` and wire the model to the client.

## Conventions

- **Crate**: `myme-services` hosts all HTTP/API and local store clients.  
- **Naming**: If **{{crateName}}** is given, use it for the module and type names (e.g. myme-foo → `foo.rs`, `FooClient`).

Reference: CLAUDE.md "Adding New Service Clients".
