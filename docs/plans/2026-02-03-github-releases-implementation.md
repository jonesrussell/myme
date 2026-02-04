# GitHub Releases Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automate GitHub releases with portable ZIP and installer when version tags are pushed.

**Architecture:** GitHub Actions workflow triggered by `v*` tags builds Rust + Qt on Windows, bundles with windeployqt, creates Inno Setup installer, and publishes both artifacts to GitHub Releases with auto-generated notes from PRs.

**Tech Stack:** GitHub Actions, Qt 6.10.1, Rust stable, Inno Setup 6, windeployqt

---

## Task 1: Create Release Notes Configuration

**Files:**
- Create: `.github/release.yml`

**Step 1: Create .github directory if needed**

Run: `mkdir -p .github`

**Step 2: Create release notes config**

Create `.github/release.yml`:

```yaml
changelog:
  exclude:
    labels:
      - ignore-for-release
  categories:
    - title: "🚀 Features"
      labels:
        - feature
        - enhancement
    - title: "🐛 Bug Fixes"
      labels:
        - bugfix
        - bug
        - fix
    - title: "📚 Documentation"
      labels:
        - docs
        - documentation
    - title: "🧹 Maintenance"
      labels:
        - chore
        - maintenance
        - dependencies
    - title: "Other Changes"
      labels:
        - "*"
```

**Step 3: Commit**

```bash
git add .github/release.yml
git commit -m "ci: add release notes configuration"
```

---

## Task 2: Create Inno Setup Installer Script

**Files:**
- Create: `installer/myme.iss`

**Step 1: Create installer directory**

Run: `mkdir -p installer`

**Step 2: Create Inno Setup script**

Create `installer/myme.iss`:

```iss
; MyMe Installer Script
; Requires Inno Setup 6.x

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=MyMe
AppVersion={#AppVersion}
AppVerName=MyMe {#AppVersion}
AppPublisher=jonesrussell
AppPublisherURL=https://github.com/jonesrussell/myme
AppSupportURL=https://github.com/jonesrussell/myme/issues
AppUpdatesURL=https://github.com/jonesrussell/myme/releases
DefaultDirName={autopf}\MyMe
DefaultGroupName=MyMe
AllowNoIcons=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=myme-{#AppVersion}-windows-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main application and all bundled files from windeployqt
Source: "..\dist\myme\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\MyMe"; Filename: "{app}\myme-qt.exe"
Name: "{group}\{cm:UninstallProgram,MyMe}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\MyMe"; Filename: "{app}\myme-qt.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\myme-qt.exe"; Description: "{cm:LaunchProgram,MyMe}"; Flags: nowait postinstall skipifsilent
```

**Step 3: Commit**

```bash
git add installer/myme.iss
git commit -m "ci: add Inno Setup installer script"
```

---

## Task 3: Create GitHub Actions Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create workflows directory if needed**

Run: `mkdir -p .github/workflows`

**Step 2: Create release workflow**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  build-windows:
    runs-on: windows-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Extract version from tag
        id: version
        shell: bash
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Install Qt
        uses: jurplel/install-qt-action@v4
        with:
          version: '6.10.1'
          host: 'windows'
          target: 'desktop'
          arch: 'win64_msvc2022_64'
          modules: 'qtquick3d'
          cache: true

      - name: Build Rust
        run: cargo build --release

      - name: Configure CMake
        run: cmake -B build -DCMAKE_BUILD_TYPE=Release

      - name: Build Qt Application
        run: cmake --build build --config Release

      - name: Create distribution directory
        run: |
          New-Item -ItemType Directory -Force -Path dist/myme
          Copy-Item build/Release/myme-qt.exe dist/myme/

      - name: Bundle Qt dependencies
        shell: cmd
        run: |
          "%Qt6_DIR%/../../../bin/windeployqt.exe" --release --qmldir crates/myme-ui/qml dist/myme/myme-qt.exe

      - name: Create portable ZIP
        run: |
          Compress-Archive -Path dist/myme/* -DestinationPath dist/myme-v${{ steps.version.outputs.VERSION }}-windows.zip

      - name: Install Inno Setup
        run: choco install innosetup -y

      - name: Build Installer
        run: |
          & "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" /DAppVersion=${{ steps.version.outputs.VERSION }} installer/myme.iss

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-release
          path: |
            dist/myme-v${{ steps.version.outputs.VERSION }}-windows.zip
            dist/myme-${{ steps.version.outputs.VERSION }}-windows-setup.exe

  release:
    needs: build-windows
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Extract version from tag
        id: version
        run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          name: windows-release
          path: dist

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            dist/myme-v${{ steps.version.outputs.VERSION }}-windows.zip
            dist/myme-${{ steps.version.outputs.VERSION }}-windows-setup.exe
          draft: false
          prerelease: ${{ contains(github.ref, '-alpha') || contains(github.ref, '-beta') || contains(github.ref, '-rc') }}
```

**Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow"
```

---

## Task 4: Create LICENSE File (if missing)

**Files:**
- Create: `LICENSE` (if not present)

**Step 1: Check if LICENSE exists**

Run: `ls LICENSE 2>/dev/null || echo "missing"`

**Step 2: Create LICENSE if missing**

If missing, create `LICENSE` with MIT license (or your preferred license):

```text
MIT License

Copyright (c) 2026 jonesrussell

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Step 3: Commit if created**

```bash
git add LICENSE
git commit -m "docs: add MIT license"
```

---

## Task 5: Create PR Labels in GitHub

**Manual step - no code**

Create these labels in GitHub repo settings (https://github.com/jonesrussell/myme/labels):

| Label | Color | Description |
|-------|-------|-------------|
| `feature` | `#0E8A16` | New feature or enhancement |
| `bugfix` | `#D73A4A` | Bug fix |
| `docs` | `#0075CA` | Documentation |
| `chore` | `#FBCA04` | Maintenance or chores |
| `ignore-for-release` | `#EEEEEE` | Exclude from release notes |

---

## Task 6: Test the Workflow Locally (Optional)

**Step 1: Verify build works**

Run locally to ensure the build succeeds before pushing:

```powershell
cargo build --release
cmake -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build --config Release
```

**Step 2: Test windeployqt locally**

```powershell
New-Item -ItemType Directory -Force -Path dist/myme
Copy-Item build/Release/myme-qt.exe dist/myme/
& "$env:Qt6_DIR/../../../bin/windeployqt.exe" --release --qmldir crates/myme-ui/qml dist/myme/myme-qt.exe
```

**Step 3: Test Inno Setup locally (if installed)**

```powershell
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" /DAppVersion=0.1.0 installer/myme.iss
```

---

## Task 7: Push and Create First Release

**Step 1: Push all changes**

```bash
git push origin main
```

**Step 2: Create and push tag**

```bash
git tag v0.1.0
git push origin v0.1.0
```

**Step 3: Monitor workflow**

Go to https://github.com/jonesrussell/myme/actions and watch the Release workflow.

Expected: ~10-15 minutes to complete, then release appears at https://github.com/jonesrussell/myme/releases

---

## Summary

| File | Purpose |
|------|---------|
| `.github/release.yml` | Maps PR labels to release note sections |
| `.github/workflows/release.yml` | Main CI/CD workflow for building and releasing |
| `installer/myme.iss` | Inno Setup installer configuration |
| `LICENSE` | License file (required by installer) |

**Release artifacts produced:**
- `myme-v{version}-windows.zip` - Portable ZIP
- `myme-{version}-windows-setup.exe` - Installer
