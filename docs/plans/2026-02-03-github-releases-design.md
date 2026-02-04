# GitHub Releases Automation Design

## Overview

Automate GitHub releases for MyMe using GitHub Actions. When a version tag is pushed (e.g., `v1.0.0`), the pipeline builds the application, creates both a portable ZIP and an installer, and publishes them as a GitHub Release with auto-generated release notes.

## Scope

**Initial scope:** Windows only (matches current development setup)

**Future expansion:** Linux and macOS can be added later using matrix builds

## Trigger Mechanism

Releases are triggered by pushing a git tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The version number is extracted from the tag and used for artifact naming.

## Release Artifacts

| Artifact | Description | Size (estimated) |
|----------|-------------|------------------|
| `myme-v{version}-windows.zip` | Portable ZIP with exe and Qt DLLs | ~50-80MB |
| `myme-v{version}-windows-setup.exe` | Inno Setup installer | ~30-50MB |

## GitHub Actions Workflow

**File:** `.github/workflows/release.yml`

**Runner:** `windows-latest`

**Steps:**

1. **Checkout** - Clone repo with full history for tag info
2. **Install Rust** - `dtolnay/rust-toolchain` with `stable`
3. **Install Qt** - `jurplel/install-qt-action` with Qt 6.10.1 (Core, Gui, Qml, Quick, QuickControls2)
4. **Cache dependencies** - Cargo registry and Qt installation
5. **Build Rust** - `cargo build --release`
6. **Build Qt app** - CMake configure and build in Release mode
7. **Bundle with windeployqt** - Copy required Qt DLLs alongside exe
8. **Create ZIP** - Package bundled folder
9. **Create Installer** - Run Inno Setup compiler
10. **Publish Release** - `softprops/action-gh-release` with auto-generated notes

## Inno Setup Installer

**File:** `installer/myme.iss`

**Features:**
- Installs to `C:\Program Files\MyMe` (user-selectable)
- Creates Start Menu shortcut
- Creates optional Desktop shortcut
- Adds uninstaller to Add/Remove Programs
- Requires admin elevation for Program Files install
- Uses lzma2 compression

**Version handling:** GitHub Actions passes the version from git tag to Inno Setup via `/D` defines.

## Release Notes Configuration

**File:** `.github/release.yml`

Auto-generates release notes from merged PRs, grouped by label:

| Label | Release Section |
|-------|-----------------|
| `feature` | 🚀 Features |
| `bugfix` | 🐛 Bug Fixes |
| `docs` | 📚 Documentation |
| `chore` | 🧹 Maintenance |

PRs without labels appear in "Other Changes."

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/release.yml` | Main release automation workflow |
| `.github/release.yml` | PR label → release notes mapping |
| `installer/myme.iss` | Inno Setup installer script |

## Code Signing

Not included in initial implementation. Can be added later without major pipeline changes. Users will see Windows SmartScreen warnings on first run.

## First Release Checklist

1. Create PR labels in GitHub repo settings: `feature`, `bugfix`, `docs`, `chore`
2. Merge the workflow PR
3. Tag and push: `git tag v0.1.0 && git push origin v0.1.0`
4. Monitor Actions tab - release appears in ~10-15 minutes

## Future Enhancements

- **Linux builds:** Add AppImage artifact using matrix strategy
- **macOS builds:** Add `.dmg` artifact (requires macOS runner, code signing)
- **Code signing:** Purchase certificate, store in GitHub Secrets
- **Automatic versioning:** Extract version from Cargo.toml instead of tag
