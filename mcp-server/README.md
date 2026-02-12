# MyMe MCP Server

MCP (Model Context Protocol) server for the MyMe repo. Exposes **tools**, **prompts**, and **resources** so Cursor and other MCP clients can run builds, tests, format QML, and follow documented workflows.

## Requirements

- Node.js 18+
- MyMe repo (workspace root must contain `Cargo.toml` and `qml.qrc`)

## Install

```bash
cd mcp-server
npm install
npm run build
```

## Configuration

- **Workspace root**: Resolved once at startup. The server uses `MYME_REPO` (env) if set, otherwise `process.cwd()`. It then searches upward for a directory containing both `Cargo.toml` and `qml.qrc`. When using Cursor, set the MCP server’s working directory to the **MyMe repo root** so the correct workspace is found.
- **QML format**: Optional. Set `QMLFORMAT_PATH` to the full path to `qmlformat` (e.g. `C:\Qt\6.10.1\msvc2022_64\bin\qmlformat.exe`). If unset, the server uses a default path on Windows or `qmlformat` on PATH elsewhere.
- **App binary**: Optional. Set `MYME_APP_PATH` to the full path to `myme-qt.exe` (or the Qt app binary). If unset, the server uses `workspace-root/build-qt/Release/myme-qt.exe` (or Debug when run in debug mode).

## Cursor setup

1. Open Cursor Settings → MCP (or the MCP servers configuration).
2. Add a new MCP server, for example:
   - **Command**: `node`
   - **Args**: `path/to/myme/mcp-server/dist/index.js`
   - **Cwd** (or env): set to the **MyMe repo root** (e.g. `c:\Users\jones\dev\myme`).

If the server is started with its working directory inside `mcp-server/`, it will still search upward and use the MyMe repo as the workspace root.

## Tools

| Tool | Description |
|------|-------------|
| `list_crates` | List workspace crate names from `Cargo.toml`. |
| `cargo_build` | Run `cargo build --release`. Optional `package` to build one crate. |
| `cargo_test` | Run `cargo test` for all workspace crates except `myme-ui`. Optional `package` to test one crate. |
| `build_qt` | Run `scripts/build.ps1` (Windows only). |
| `run_app` | Run the MyMe Qt app. Optional `mode` (debug/release), `env` overrides, `args`. |
| `qml_format` | Format QML files. Pass `paths` (relative to workspace root). |

All tools return a normalized envelope: `success`, `exitCode`, `stdout`, `stderr`, optional `message`.

## Prompts

| Prompt | Arguments | Description |
|--------|------------|-------------|
| `add_new_ui_page` | `pageName` | Step-by-step checklist to add a new QML page and cxx-qt model. |
| `add_new_service_client` | `crateName` (optional) | Checklist to add a new service client in myme-services. |
| `add_oauth_provider` | `providerName` | Checklist to add a new OAuth provider in myme-auth. |

Templates live in `prompts/*.md` and are parameterized (e.g. `{{pageName}}`, `{{providerName}}`).

## Resources

| URI | Description |
|-----|-------------|
| `myme://theme` | Theme colors and typography (curated from Theme.qml). |
| `myme://pages` | List of QML pages and models. |
| `myme://project-context` | Short project summary and reminders. |
| `myme://version` | Server version, repo git hash, Node version. |

Curated content is in `resources/*.md`. Update `theme-reference.md` and `pages-checklist.md` when Theme.qml or pages/models change. A generator script can be added later to refresh these from the repo.

## Logging

The server logs to stderr: tool invocations, exit codes, resource reads, and errors. Use these logs when debugging Cursor integration or failed builds.
