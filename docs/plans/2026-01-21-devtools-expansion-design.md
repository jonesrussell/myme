# Dev Tools Expansion Design

**Date:** 2026-01-21
**Branch:** `feature/devtools-expansion`
**Status:** Approved

## Overview

Implement 5 new developer tools for the Dev Tools page, expanding on the existing JWT Generator pattern. All tools use Rust models with cxx-qt bridges exposed to QML.

## Tools Summary

| Tool | Description | Category |
|------|-------------|----------|
| Encoding Hub | Base64, Hex, URL encoding/decoding | Encoding |
| UUID Generator | V1, V4, V5, V7 with bulk generation and format options | Generators |
| JSON Toolkit | Format, validate, minify, tree view, diff, YAML/TOML conversion | Formatting |
| Hash Generator | MD5, SHA-1, SHA-256, SHA-512, file hashing, HMAC, verification | Security |
| Time Toolkit | Unix timestamps, ISO 8601, RFC 2822, timezones, date arithmetic | Conversion |

## Architecture

### File Structure

```
crates/myme-ui/src/models/
├── mod.rs                    # Add new model exports
├── jwt_model.rs              # Existing
├── encoding_model.rs         # NEW: Encoding Hub
├── uuid_model.rs             # NEW: UUID Generator
├── json_model.rs             # NEW: JSON Toolkit
├── hash_model.rs             # NEW: Hash Generator
├── time_model.rs             # NEW: Time Toolkit

crates/myme-ui/qml/pages/
└── DevToolsPage.qml          # Add 5 new tool components
```

### Bridge Pattern

Each tool follows the existing `JwtModel` pattern:
- Rust struct with `#[cxx_qt::bridge]` macro
- Properties exposed via `#[qproperty]` for reactive QML binding
- Methods exposed via `#[qinvokable]` for QML to call
- Results stored in properties (not returned directly)

### No Async Needed

These tools are CPU-bound transformations. Direct synchronous execution is sufficient.

---

## Tool 1: Encoding Hub

### Model Properties

```rust
// encoding_model.rs
- input: QString           // User's input text
- output: QString          // Encoded/decoded result
- encoding_type: QString   // "base64", "base64url", "hex", "url"
- mode: QString            // "encode" or "decode"
- error_message: QString   // Validation/decode errors
```

### Methods

```rust
#[qinvokable]
fn process(&self)          // Encode or decode based on mode + encoding_type

#[qinvokable]
fn swap(&self)             // Swap input ↔ output (quick re-encode)

#[qinvokable]
fn clear(&self)            // Reset all fields
```

### UI Layout

```
┌─────────────────────────────────────────────────────┐
│  Encoding Type: [Base64] [Base64 URL] [Hex] [URL]   │
├─────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐    │
│  │ Input                                       │    │
│  │ [Multi-line text area]                      │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│        [↓ Encode]    [↑ Decode]    [⇅ Swap]        │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │ Output                                      │    │
│  │ [Multi-line text area, read-only]           │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  [Copy to Clipboard]                    Copied! ✓   │
└─────────────────────────────────────────────────────┘
```

---

## Tool 2: UUID Generator

### Model Properties

```rust
// uuid_model.rs
- version: QString              // "v1", "v4", "v5", "v7"
- generated_uuids: QStringList  // List of generated UUIDs
- count: i32                    // How many to generate (bulk)
- format: QString               // "standard", "no-dashes", "uppercase", "braces"

// V5 namespace options
- namespace_type: QString       // "dns", "url", "oid", "x500", "custom"
- custom_namespace: QString     // Custom namespace UUID
- name: QString                 // Name to hash for v5
```

### Methods

```rust
#[qinvokable]
fn generate(&self)              // Generate UUID(s) based on version + count

#[qinvokable]
fn copy_all(&self)              // Copy all generated UUIDs to clipboard

#[qinvokable]
fn clear(&self)                 // Clear generated list
```

### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Version: [V1] [V4] [V5] [V7]                            │
├──────────────────────────────────────────────────────────┤
│  ┌─ V5 Options (shown when V5 selected) ───────────────┐ │
│  │ Namespace: [DNS ▾]  Name: [___________________]     │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  Count: [1  ] [▲▼]     Format: [Standard ▾]             │
│                                                          │
│  [Generate]                                              │
├──────────────────────────────────────────────────────────┤
│  Generated UUIDs:                                        │
│  ┌─────────────────────────────────────────────────────┐ │
│  │ 550e8400-e29b-41d4-a716-446655440000           [📋] │ │
│  │ 6ba7b810-9dad-11d1-80b4-00c04fd430c8           [📋] │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                          │
│  [Copy All]                                              │
└──────────────────────────────────────────────────────────┘
```

### Version Notes

- **V1**: MAC address + timestamp (uses random node ID for privacy)
- **V4**: Cryptographically random
- **V5**: SHA-1 hash of namespace + name (deterministic)
- **V7**: Unix timestamp + random (sortable, modern replacement for v1)

---

## Tool 3: JSON Toolkit

### Model Properties

```rust
// json_model.rs
- input: QString              // Raw JSON input
- output: QString             // Formatted/minified/converted output
- error_message: QString      // Parse errors with line/column
- error_line: i32             // Line number of error (-1 if none)

// Formatting options
- indent_size: i32            // 2, 4, or tab
- indent_type: QString        // "spaces" or "tabs"
- sort_keys: bool             // Alphabetically sort object keys

// JSON Path
- json_path_query: QString    // e.g., "$.store.book[*].author"
- json_path_result: QString   // Query result

// Conversion
- output_format: QString      // "json", "yaml", "toml"

// Tree view
- tree_json: QString          // Structured JSON for tree rendering

// Diff
- diff_input_a: QString       // First JSON for comparison
- diff_input_b: QString       // Second JSON for comparison
- diff_result: QString        // Diff output
```

### Methods

```rust
#[qinvokable]
fn format(&self)              // Pretty-print with current options

#[qinvokable]
fn minify(&self)              // Remove all whitespace

#[qinvokable]
fn validate(&self) -> bool    // Check if valid JSON

#[qinvokable]
fn query_path(&self)          // Execute JSON path query

#[qinvokable]
fn convert(&self)             // Convert to output_format (YAML/TOML)

#[qinvokable]
fn compute_diff(&self)        // Compare diff_input_a vs diff_input_b
```

### UI Layout (Tabbed)

```
┌─────────────────────────────────────────────────────────────┐
│  [Format] [Tree View] [JSON Path] [Convert] [Diff]          │
├─────────────────────────────────────────────────────────────┤
│  FORMAT TAB:                                                │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Input JSON                                             │ │
│  │ [Multi-line with syntax highlighting]                  │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                             │
│  Indent: [2 spaces ▾]  [✓] Sort keys                       │
│  [Format]  [Minify]  [Validate]                            │
│                                                             │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Output                                                 │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Tool 4: Hash Generator

### Model Properties

```rust
// hash_model.rs
- input: QString              // Text input to hash
- input_type: QString         // "text" or "file"
- file_path: QString          // Path when input_type = "file"
- file_name: QString          // Display name for file
- file_size: i64              // File size in bytes

// Hash results (computed simultaneously)
- hash_md5: QString
- hash_sha1: QString
- hash_sha256: QString
- hash_sha512: QString

// HMAC
- hmac_enabled: bool
- hmac_key: QString
- hmac_algorithm: QString     // "sha256", "sha512"
- hmac_result: QString

// Comparison
- compare_mode: bool
- compare_hash: QString       // Hash to compare against
- compare_result: QString     // "match", "no-match", or ""

// Status
- is_processing: bool         // For large files
- error_message: QString
```

### Methods

```rust
#[qinvokable]
fn hash_text(&self)           // Hash the text input

#[qinvokable]
fn hash_file(&self, path: QString)  // Hash file at path

#[qinvokable]
fn compute_hmac(&self)        // Compute HMAC with key

#[qinvokable]
fn compare(&self)             // Compare hash against compare_hash

#[qinvokable]
fn export_checksums(&self) -> QString  // Generate checksum file content

#[qinvokable]
fn clear(&self)
```

### UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│  Input: [Text] [File]                                        │
├──────────────────────────────────────────────────────────────┤
│  ┌─ Text Input ────────────────────────────────────────────┐ │
│  │ [Multi-line text area]                                  │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  [Hash]                                                      │
├──────────────────────────────────────────────────────────────┤
│  Results:                              [Copy All] [Export]   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ MD5:    d41d8cd98f00b204e9800998ecf8427e            [📋]│ │
│  │ SHA-1:  da39a3ee5e6b4b0d3255bfef95601890afd80709    [📋]│ │
│  │ SHA-256: e3b0c44298fc1c149afbf4c8996fb924...        [📋]│ │
│  │ SHA-512: cf83e1357eefb8bdf1542850d66d8007...        [📋]│ │
│  └─────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  ┌─ HMAC (collapsible) ────────────────────────────────────┐ │
│  │ Key: [____________]  Algorithm: [SHA-256 ▾]  [Compute]  │ │
│  │ Result: 5d5d139563c95b5967b9bd9a8c9b...                  │ │
│  └─────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  ┌─ Verify Hash (collapsible) ─────────────────────────────┐ │
│  │ Expected: [paste hash to verify________________] [Check] │ │
│  │ ✓ Match! (SHA-256)                                      │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

---

## Tool 5: Time Toolkit

### Model Properties

```rust
// time_model.rs
- input_value: QString        // User input (timestamp or date string)
- input_type: QString         // "unix_s", "unix_ms", "unix_us", "unix_ns", "iso8601", "rfc2822", "custom"
- custom_format: QString      // Custom strftime format string

// Timezone
- timezone: QString           // e.g., "UTC", "America/New_York", "local"
- available_timezones: QStringList  // List for dropdown

// Output (all formats computed at once)
- unix_seconds: i64
- unix_milliseconds: i64
- unix_microseconds: i64
- unix_nanoseconds: i64
- iso8601: QString
- rfc2822: QString
- human_readable: QString
- relative_time: QString
- custom_output: QString

// Date arithmetic
- arithmetic_value: i32
- arithmetic_unit: QString    // "seconds", "minutes", "hours", "days", "weeks", "months", "years"

// Status
- error_message: QString
- is_valid: bool
```

### Methods

```rust
#[qinvokable]
fn parse(&self)               // Parse input_value, populate all outputs

#[qinvokable]
fn set_now(&self)             // Set input to current time

#[qinvokable]
fn add_time(&self)            // Add arithmetic_value + arithmetic_unit

#[qinvokable]
fn subtract_time(&self)       // Subtract arithmetic_value + arithmetic_unit

#[qinvokable]
fn copy_format(&self, format: QString)  // Copy specific format to clipboard
```

### UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│  Input                                              [Now]    │
├──────────────────────────────────────────────────────────────┤
│  Type: [Unix (seconds) ▾]    Timezone: [UTC ▾]              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ 1705315800                                              │ │
│  └─────────────────────────────────────────────────────────┘ │
│  [Parse]                                                     │
├──────────────────────────────────────────────────────────────┤
│  All Formats:                                                │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Unix (s):    1705315800                             [📋]│ │
│  │ Unix (ms):   1705315800000                          [📋]│ │
│  │ Unix (μs):   1705315800000000                       [📋]│ │
│  │ Unix (ns):   1705315800000000000                    [📋]│ │
│  │ ISO 8601:    2024-01-15T10:30:00Z                   [📋]│ │
│  │ RFC 2822:    Mon, 15 Jan 2024 10:30:00 +0000        [📋]│ │
│  │ Human:       January 15, 2024 10:30:00 AM           [📋]│ │
│  │ Relative:    11 months ago                          [📋]│ │
│  └─────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  ┌─ Custom Format ─────────────────────────────────────────┐ │
│  │ Format: [%Y-%m-%d %H:%M:%S____]  → 2024-01-15 10:30:00  │ │
│  └─────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  ┌─ Date Arithmetic ───────────────────────────────────────┐ │
│  │ [+] [-]  [7  ]  [days ▾]  →  2024-01-22T10:30:00Z       │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

---

## Dependencies

**New in crates/myme-ui/Cargo.toml:**

```toml
# Encoding
base64 = "0.22"
hex = "0.4"
percent-encoding = "2.3"

# UUID
uuid = { version = "1.0", features = ["v1", "v4", "v5", "v7"] }

# JSON/YAML/TOML
serde_yaml = "0.9"
toml = "0.8"
jsonpath-rust = "0.5"

# Hashing
md-5 = "0.10"
sha1 = "0.10"
sha2 = "0.10"
hmac = "0.12"
digest = "0.10"

# Time
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"
```

---

## Implementation Order

1. **Encoding Hub** - Simplest, establishes pattern
2. **UUID Generator** - Straightforward, no complex UI
3. **Hash Generator** - Medium complexity, file handling
4. **Time Toolkit** - Medium complexity, timezone handling
5. **JSON Toolkit** - Most complex (tabs, tree view, diff)

---

## DevToolsPage.qml Updates

1. Update tool definitions with new names, remove `comingSoon` flags
2. Add model instantiations for each new tool
3. Add 5 new `Component` blocks for tool UIs
4. Update `toolLoader.sourceComponent` switch statement
