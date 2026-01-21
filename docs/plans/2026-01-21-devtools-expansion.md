# Dev Tools Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement 5 full-featured developer tools (Encoding Hub, UUID Generator, JSON Toolkit, Hash Generator, Time Toolkit) for the Dev Tools page.

**Architecture:** Each tool follows the existing JwtModel pattern: a Rust struct with `#[cxx_qt::bridge]` exposing properties via `#[qproperty]` and methods via `#[qinvokable]` to QML. Tools are synchronous (no async needed).

**Tech Stack:** Rust, cxx-qt 0.8, Qt 6/QML, base64, hex, percent-encoding, uuid, serde_json, serde_yaml, toml, jsonpath-rust, md-5, sha1, sha2, hmac, chrono, chrono-tz

---

## Phase 1: Dependencies & Setup

### Task 1.1: Add Cargo Dependencies

**Files:**
- Modify: `crates/myme-ui/Cargo.toml`

**Step 1: Add encoding dependencies**

Add after line 30 (after `dirs = "5.0"`):

```toml
# Encoding tools
base64 = "0.22"
hex = "0.4"
percent-encoding = "2.3"

# UUID generation
uuid = { version = "1.11", features = ["v1", "v4", "v5", "v7"] }

# JSON/YAML/TOML conversion
serde_yaml = "0.9"
toml = "0.8"
jsonpath-rust = "0.7"

# Hashing
md-5 = "0.10"
sha1 = "0.10"
sha2 = "0.10"
hmac = "0.12"
digest = "0.10"

# Time handling
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.10"
```

**Step 2: Verify dependencies resolve**

Run: `cargo check -p myme-ui`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/myme-ui/Cargo.toml
git commit -m "chore: add dependencies for dev tools expansion"
```

---

## Phase 2: Encoding Hub

### Task 2.1: Create Encoding Model

**Files:**
- Create: `crates/myme-ui/src/models/encoding_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

**Step 1: Create the encoding model file**

Create `crates/myme-ui/src/models/encoding_model.rs`:

```rust
use core::pin::Pin;

use base64::{engine::general_purpose, Engine};
use cxx_qt_lib::QString;
use percent_encoding::{percent_decode_str, percent_encode, NON_ALPHANUMERIC};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, input)]
        #[qproperty(QString, output)]
        #[qproperty(QString, encoding_type)]
        #[qproperty(QString, error_message)]
        type EncodingModel = super::EncodingModelRust;

        #[qinvokable]
        fn encode(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn decode(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn swap(self: Pin<&mut EncodingModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut EncodingModel>);
    }
}

#[derive(Default)]
pub struct EncodingModelRust {
    input: QString,
    output: QString,
    encoding_type: QString,
    error_message: QString,
}

impl qobject::EncodingModel {
    pub fn encode(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        self.as_mut().set_output(QString::from(""));

        let input_str = self.as_ref().input().to_string();
        let encoding = self.as_ref().encoding_type().to_string();

        if input_str.is_empty() {
            return;
        }

        let result = match encoding.as_str() {
            "base64" => general_purpose::STANDARD.encode(input_str.as_bytes()),
            "base64url" => general_purpose::URL_SAFE.encode(input_str.as_bytes()),
            "hex" => hex::encode(input_str.as_bytes()),
            "url" => percent_encode(input_str.as_bytes(), NON_ALPHANUMERIC).to_string(),
            _ => general_purpose::STANDARD.encode(input_str.as_bytes()),
        };

        self.as_mut().set_output(QString::from(&result));
    }

    pub fn decode(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        self.as_mut().set_output(QString::from(""));

        let input_str = self.as_ref().input().to_string();
        let encoding = self.as_ref().encoding_type().to_string();

        if input_str.is_empty() {
            return;
        }

        let result = match encoding.as_str() {
            "base64" => match general_purpose::STANDARD.decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Base64: {}", e)),
            },
            "base64url" => match general_purpose::URL_SAFE.decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Base64 URL: {}", e)),
            },
            "hex" => match hex::decode(input_str.trim()) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(format!("Invalid UTF-8: {}", e)),
                },
                Err(e) => Err(format!("Invalid Hex: {}", e)),
            },
            "url" => match percent_decode_str(input_str.trim()).decode_utf8() {
                Ok(s) => Ok(s.to_string()),
                Err(e) => Err(format!("Invalid URL encoding: {}", e)),
            },
            _ => Err("Unknown encoding type".to_string()),
        };

        match result {
            Ok(decoded) => self.as_mut().set_output(QString::from(&decoded)),
            Err(e) => self.as_mut().set_error_message(QString::from(&e)),
        }
    }

    pub fn swap(mut self: Pin<&mut Self>) {
        let current_output = self.as_ref().output().to_string();
        self.as_mut().set_input(QString::from(&current_output));
        self.as_mut().set_output(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input(QString::from(""));
        self.as_mut().set_output(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
```

**Step 2: Add to mod.rs**

Add to `crates/myme-ui/src/models/mod.rs`:

```rust
pub mod encoding_model;
```

**Step 3: Add to build.rs**

Add `.file("src/models/encoding_model.rs")` to `crates/myme-ui/build.rs` after the weather_model line.

**Step 4: Verify compilation**

Run: `cargo build -p myme-ui`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/myme-ui/src/models/encoding_model.rs crates/myme-ui/src/models/mod.rs crates/myme-ui/build.rs
git commit -m "feat(devtools): add EncodingModel for Base64/Hex/URL encoding"
```

### Task 2.2: Add Encoding Hub UI to DevToolsPage

**Files:**
- Modify: `crates/myme-ui/qml/pages/DevToolsPage.qml`

**Step 1: Update tool definition**

In DevToolsPage.qml, update the base64 tool entry (around line 25-31):

```qml
{
    id: "encoding",
    name: "Encoding Hub",
    description: "Encode and decode Base64, Hex, and URL strings",
    icon: Icons.code,
    category: "Encoding"
},
```

**Step 2: Add EncodingModel instantiation**

After the JwtModel instantiation (around line 88), add:

```qml
EncodingModel {
    id: encodingModel
    encoding_type: "base64"
}
```

**Step 3: Add encoding tool component**

After jwtToolComponent (around line 734), add:

```qml
// Encoding Hub Component
Component {
    id: encodingToolComponent

    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingLg

            // Encoding type selector
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: encodingTypeRow.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                radius: Theme.cardRadius

                RowLayout {
                    id: encodingTypeRow
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingSm

                    Label {
                        text: "Type:"
                        color: Theme.text
                        font.bold: true
                    }

                    Repeater {
                        model: [
                            { id: "base64", label: "Base64" },
                            { id: "base64url", label: "Base64 URL" },
                            { id: "hex", label: "Hex" },
                            { id: "url", label: "URL" }
                        ]

                        Button {
                            text: modelData.label
                            checkable: true
                            checked: encodingModel.encoding_type === modelData.id
                            onClicked: encodingModel.encoding_type = modelData.id

                            background: Rectangle {
                                radius: Theme.buttonRadius
                                color: parent.checked ? Theme.primary : (parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt)
                            }

                            contentItem: Text {
                                text: parent.text
                                color: parent.checked ? Theme.primaryText : Theme.text
                                font.pixelSize: Theme.fontSizeSmall
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                            }
                        }
                    }
                }
            }

            // Error banner
            Rectangle {
                visible: encodingModel.error_message.length > 0
                Layout.fillWidth: true
                Layout.preferredHeight: 50
                color: Theme.errorBg
                border.color: Theme.error
                radius: Theme.cardRadius

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: Icons.warning
                        font.family: Icons.family
                        font.pixelSize: 20
                        color: Theme.error
                    }

                    Label {
                        text: encodingModel.error_message
                        color: Theme.error
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                    }
                }
            }

            // Input section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 200
                color: Theme.surface
                border.color: Theme.border
                radius: Theme.cardRadius

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingSm

                    Label {
                        text: "Input"
                        font.bold: true
                        color: Theme.text
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        TextArea {
                            id: encodingInput
                            text: encodingModel.input
                            placeholderText: "Enter text to encode or decode..."
                            wrapMode: TextEdit.Wrap
                            font.family: "Consolas, Monaco, monospace"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.text
                            placeholderTextColor: Theme.textMuted
                            onTextChanged: encodingModel.input = text

                            background: Rectangle {
                                color: Theme.inputBg
                                radius: Theme.inputRadius
                            }
                        }
                    }
                }
            }

            // Action buttons
            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: Theme.spacingMd

                Button {
                    text: Icons.arrowDown + " Encode"
                    font.family: Icons.family
                    onClicked: encodingModel.encode()

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.primaryHover : Theme.primary
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.primaryText
                        font.pixelSize: Theme.fontSizeNormal
                        font.bold: true
                        horizontalAlignment: Text.AlignHCenter
                    }
                }

                Button {
                    text: Icons.arrowUp + " Decode"
                    font.family: Icons.family
                    onClicked: encodingModel.decode()

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                        border.color: Theme.border
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.text
                        font.pixelSize: Theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                    }
                }

                Button {
                    text: Icons.arrowsClockwise + " Swap"
                    font.family: Icons.family
                    onClicked: encodingModel.swap()

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                        border.color: Theme.border
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.text
                        font.pixelSize: Theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                    }
                }

                Button {
                    text: "Clear"
                    onClicked: {
                        encodingModel.clear()
                        encodingInput.text = ""
                    }

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                        border.color: Theme.border
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                    }
                }
            }

            // Output section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 200
                color: Theme.surface
                border.color: encodingModel.output.length > 0 ? Theme.success : Theme.border
                border.width: encodingModel.output.length > 0 ? 2 : 1
                radius: Theme.cardRadius

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingSm

                    RowLayout {
                        Layout.fillWidth: true

                        Label {
                            text: "Output"
                            font.bold: true
                            color: Theme.text
                            Layout.fillWidth: true
                        }

                        Button {
                            visible: encodingModel.output.length > 0
                            text: "Copy"
                            onClicked: {
                                encodingOutput.selectAll()
                                encodingOutput.copy()
                                encodingOutput.deselect()
                            }

                            background: Rectangle {
                                radius: Theme.buttonRadius
                                color: parent.hovered ? Theme.surfaceHover : "transparent"
                            }

                            contentItem: Text {
                                text: parent.text
                                color: Theme.primary
                                font.pixelSize: Theme.fontSizeSmall
                            }
                        }
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        TextArea {
                            id: encodingOutput
                            text: encodingModel.output
                            readOnly: true
                            wrapMode: TextEdit.WrapAnywhere
                            font.family: "Consolas, Monaco, monospace"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.text
                            selectByMouse: true

                            background: Rectangle {
                                color: Theme.inputBg
                                radius: Theme.inputRadius
                            }
                        }
                    }
                }
            }

            Item { Layout.fillHeight: true }
        }
    }
}
```

**Step 4: Update toolLoader sourceComponent**

Update the toolLoader sourceComponent (around line 375-379) to include encoding:

```qml
sourceComponent: {
    if (currentTool === "jwt") return jwtToolComponent;
    if (currentTool === "encoding") return encodingToolComponent;
    return null;
}
```

**Step 5: Verify QML syntax**

Run: `/mnt/c/Qt/6.10.1/msvc2022_64/bin/qmlformat.exe -i crates/myme-ui/qml/pages/DevToolsPage.qml`

**Step 6: Build and test**

Run: `cargo build -p myme-ui && cd build-qt && cmake --build . --config Release`
Expected: Builds successfully

**Step 7: Commit**

```bash
git add crates/myme-ui/qml/pages/DevToolsPage.qml
git commit -m "feat(devtools): add Encoding Hub UI component"
```

---

## Phase 3: UUID Generator

### Task 3.1: Create UUID Model

**Files:**
- Create: `crates/myme-ui/src/models/uuid_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

**Step 1: Create the UUID model file**

Create `crates/myme-ui/src/models/uuid_model.rs`:

```rust
use core::pin::Pin;

use cxx_qt_lib::{QList, QString, QStringList};
use uuid::{Uuid, uuid};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qstringlist.h");
        type QString = cxx_qt_lib::QString;
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, version)]
        #[qproperty(i32, count)]
        #[qproperty(QString, format)]
        #[qproperty(QString, namespace_type)]
        #[qproperty(QString, custom_namespace)]
        #[qproperty(QString, name)]
        #[qproperty(QStringList, generated_uuids)]
        #[qproperty(QString, error_message)]
        type UuidModel = super::UuidModelRust;

        #[qinvokable]
        fn generate(self: Pin<&mut UuidModel>);

        #[qinvokable]
        fn clear(self: Pin<&mut UuidModel>);
    }
}

pub struct UuidModelRust {
    version: QString,
    count: i32,
    format: QString,
    namespace_type: QString,
    custom_namespace: QString,
    name: QString,
    generated_uuids: QStringList,
    error_message: QString,
}

impl Default for UuidModelRust {
    fn default() -> Self {
        Self {
            version: QString::from("v4"),
            count: 1,
            format: QString::from("standard"),
            namespace_type: QString::from("dns"),
            custom_namespace: QString::from(""),
            name: QString::from(""),
            generated_uuids: QStringList::default(),
            error_message: QString::from(""),
        }
    }
}

impl qobject::UuidModel {
    pub fn generate(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));

        let version = self.as_ref().version().to_string();
        let count = self.as_ref().count().max(1).min(100);
        let format = self.as_ref().format().to_string();
        let namespace_type = self.as_ref().namespace_type().to_string();
        let custom_ns = self.as_ref().custom_namespace().to_string();
        let name = self.as_ref().name().to_string();

        let mut uuids = QStringList::default();

        for _ in 0..count {
            let uuid_result = match version.as_str() {
                "v1" => Ok(Uuid::now_v1(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55])),
                "v4" => Ok(Uuid::new_v4()),
                "v5" => {
                    if name.is_empty() {
                        Err("Name is required for UUID v5".to_string())
                    } else {
                        let ns = match namespace_type.as_str() {
                            "dns" => Uuid::NAMESPACE_DNS,
                            "url" => Uuid::NAMESPACE_URL,
                            "oid" => Uuid::NAMESPACE_OID,
                            "x500" => Uuid::NAMESPACE_X500,
                            "custom" => {
                                match Uuid::parse_str(&custom_ns) {
                                    Ok(u) => u,
                                    Err(e) => return self.as_mut().set_error_message(
                                        QString::from(&format!("Invalid custom namespace UUID: {}", e))
                                    ),
                                }
                            }
                            _ => Uuid::NAMESPACE_DNS,
                        };
                        Ok(Uuid::new_v5(&ns, name.as_bytes()))
                    }
                }
                "v7" => Ok(Uuid::now_v7()),
                _ => Ok(Uuid::new_v4()),
            };

            match uuid_result {
                Ok(u) => {
                    let formatted = Self::format_uuid(&u, &format);
                    uuids.append(QString::from(&formatted));
                }
                Err(e) => {
                    self.as_mut().set_error_message(QString::from(&e));
                    return;
                }
            }
        }

        self.as_mut().set_generated_uuids(uuids);
    }

    fn format_uuid(uuid: &Uuid, format: &str) -> String {
        match format {
            "no-dashes" => uuid.simple().to_string(),
            "uppercase" => uuid.to_string().to_uppercase(),
            "braces" => format!("{{{}}}", uuid),
            _ => uuid.to_string(),
        }
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_generated_uuids(QStringList::default());
        self.as_mut().set_error_message(QString::from(""));
    }
}
```

**Step 2: Add to mod.rs**

Add to `crates/myme-ui/src/models/mod.rs`:

```rust
pub mod uuid_model;
```

**Step 3: Add to build.rs**

Add `.file("src/models/uuid_model.rs")` to `crates/myme-ui/build.rs`.

**Step 4: Verify compilation**

Run: `cargo build -p myme-ui`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add crates/myme-ui/src/models/uuid_model.rs crates/myme-ui/src/models/mod.rs crates/myme-ui/build.rs
git commit -m "feat(devtools): add UuidModel for UUID generation"
```

### Task 3.2: Add UUID Generator UI

**Files:**
- Modify: `crates/myme-ui/qml/pages/DevToolsPage.qml`

**Step 1: Update uuid tool definition**

Update the uuid tool entry to remove comingSoon:

```qml
{
    id: "uuid",
    name: "UUID Generator",
    description: "Generate UUIDs v1, v4, v5, v7 with format options",
    icon: Icons.squaresFour,
    category: "Generators"
},
```

**Step 2: Add UuidModel instantiation**

After EncodingModel:

```qml
UuidModel {
    id: uuidModel
}
```

**Step 3: Add UUID tool component**

After encodingToolComponent, add the uuidToolComponent (see design doc for full UI layout).

**Step 4: Update toolLoader**

Add: `if (currentTool === "uuid") return uuidToolComponent;`

**Step 5: Build and test**

Run: `cargo build -p myme-ui && cd build-qt && cmake --build . --config Release`

**Step 6: Commit**

```bash
git add crates/myme-ui/qml/pages/DevToolsPage.qml
git commit -m "feat(devtools): add UUID Generator UI component"
```

---

## Phase 4: Hash Generator

### Task 4.1: Create Hash Model

**Files:**
- Create: `crates/myme-ui/src/models/hash_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

**Step 1: Create the hash model file**

Create `crates/myme-ui/src/models/hash_model.rs`:

```rust
use core::pin::Pin;

use cxx_qt_lib::QString;
use digest::Digest;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, input)]
        #[qproperty(QString, input_type)]
        #[qproperty(QString, file_path)]
        #[qproperty(QString, file_name)]
        #[qproperty(i64, file_size)]
        #[qproperty(QString, hash_md5)]
        #[qproperty(QString, hash_sha1)]
        #[qproperty(QString, hash_sha256)]
        #[qproperty(QString, hash_sha512)]
        #[qproperty(bool, hmac_enabled)]
        #[qproperty(QString, hmac_key)]
        #[qproperty(QString, hmac_algorithm)]
        #[qproperty(QString, hmac_result)]
        #[qproperty(QString, compare_hash)]
        #[qproperty(QString, compare_result)]
        #[qproperty(QString, error_message)]
        type HashModel = super::HashModelRust;

        #[qinvokable]
        fn hash_text(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn hash_file(self: Pin<&mut HashModel>, path: QString);

        #[qinvokable]
        fn compute_hmac(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn compare(self: Pin<&mut HashModel>);

        #[qinvokable]
        fn export_checksums(self: &HashModel) -> QString;

        #[qinvokable]
        fn clear(self: Pin<&mut HashModel>);
    }
}

pub struct HashModelRust {
    input: QString,
    input_type: QString,
    file_path: QString,
    file_name: QString,
    file_size: i64,
    hash_md5: QString,
    hash_sha1: QString,
    hash_sha256: QString,
    hash_sha512: QString,
    hmac_enabled: bool,
    hmac_key: QString,
    hmac_algorithm: QString,
    hmac_result: QString,
    compare_hash: QString,
    compare_result: QString,
    error_message: QString,
}

impl Default for HashModelRust {
    fn default() -> Self {
        Self {
            input: QString::from(""),
            input_type: QString::from("text"),
            file_path: QString::from(""),
            file_name: QString::from(""),
            file_size: 0,
            hash_md5: QString::from(""),
            hash_sha1: QString::from(""),
            hash_sha256: QString::from(""),
            hash_sha512: QString::from(""),
            hmac_enabled: false,
            hmac_key: QString::from(""),
            hmac_algorithm: QString::from("sha256"),
            hmac_result: QString::from(""),
            compare_hash: QString::from(""),
            compare_result: QString::from(""),
            error_message: QString::from(""),
        }
    }
}

impl qobject::HashModel {
    pub fn hash_text(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();

        if input.is_empty() {
            self.as_mut().clear_hashes();
            return;
        }

        let bytes = input.as_bytes();
        self.as_mut().compute_all_hashes(bytes);
    }

    pub fn hash_file(mut self: Pin<&mut Self>, path: QString) {
        self.as_mut().set_error_message(QString::from(""));
        let path_str = path.to_string();

        match std::fs::read(&path_str) {
            Ok(bytes) => {
                let file_name = std::path::Path::new(&path_str)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                self.as_mut().set_file_path(path);
                self.as_mut().set_file_name(QString::from(&file_name));
                self.as_mut().set_file_size(bytes.len() as i64);
                self.as_mut().compute_all_hashes(&bytes);
            }
            Err(e) => {
                self.as_mut().set_error_message(QString::from(&format!("Failed to read file: {}", e)));
            }
        }
    }

    fn compute_all_hashes(mut self: Pin<&mut Self>, bytes: &[u8]) {
        let md5 = hex::encode(Md5::digest(bytes));
        let sha1 = hex::encode(Sha1::digest(bytes));
        let sha256 = hex::encode(Sha256::digest(bytes));
        let sha512 = hex::encode(Sha512::digest(bytes));

        self.as_mut().set_hash_md5(QString::from(&md5));
        self.as_mut().set_hash_sha1(QString::from(&sha1));
        self.as_mut().set_hash_sha256(QString::from(&sha256));
        self.as_mut().set_hash_sha512(QString::from(&sha512));
    }

    fn clear_hashes(mut self: Pin<&mut Self>) {
        self.as_mut().set_hash_md5(QString::from(""));
        self.as_mut().set_hash_sha1(QString::from(""));
        self.as_mut().set_hash_sha256(QString::from(""));
        self.as_mut().set_hash_sha512(QString::from(""));
    }

    pub fn compute_hmac(mut self: Pin<&mut Self>) {
        self.as_mut().set_error_message(QString::from(""));
        let input = self.as_ref().input().to_string();
        let key = self.as_ref().hmac_key().to_string();
        let algorithm = self.as_ref().hmac_algorithm().to_string();

        if input.is_empty() || key.is_empty() {
            self.as_mut().set_hmac_result(QString::from(""));
            return;
        }

        let result = match algorithm.as_str() {
            "sha256" => {
                let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(input.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            "sha512" => {
                let mut mac = Hmac::<Sha512>::new_from_slice(key.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(input.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            }
            _ => return,
        };

        self.as_mut().set_hmac_result(QString::from(&result));
    }

    pub fn compare(mut self: Pin<&mut Self>) {
        let compare = self.as_ref().compare_hash().to_string().to_lowercase().replace(" ", "");

        if compare.is_empty() {
            self.as_mut().set_compare_result(QString::from(""));
            return;
        }

        let hashes = [
            (self.as_ref().hash_md5().to_string(), "MD5"),
            (self.as_ref().hash_sha1().to_string(), "SHA-1"),
            (self.as_ref().hash_sha256().to_string(), "SHA-256"),
            (self.as_ref().hash_sha512().to_string(), "SHA-512"),
        ];

        for (hash, name) in hashes {
            if hash.to_lowercase() == compare {
                self.as_mut().set_compare_result(QString::from(&format!("match:{}", name)));
                return;
            }
        }

        self.as_mut().set_compare_result(QString::from("no-match"));
    }

    pub fn export_checksums(&self) -> QString {
        let mut output = String::new();
        let name = self.file_name().to_string();
        let name = if name.is_empty() { "input.txt" } else { &name };

        let sha256 = self.hash_sha256().to_string();
        if !sha256.is_empty() {
            output.push_str(&format!("{}  {}\n", sha256, name));
        }

        QString::from(&output)
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        self.as_mut().set_input(QString::from(""));
        self.as_mut().set_file_path(QString::from(""));
        self.as_mut().set_file_name(QString::from(""));
        self.as_mut().set_file_size(0);
        self.as_mut().clear_hashes();
        self.as_mut().set_hmac_result(QString::from(""));
        self.as_mut().set_compare_hash(QString::from(""));
        self.as_mut().set_compare_result(QString::from(""));
        self.as_mut().set_error_message(QString::from(""));
    }
}
```

**Step 2-5:** Same pattern as previous models (mod.rs, build.rs, verify, commit)

```bash
git commit -m "feat(devtools): add HashModel for text and file hashing"
```

### Task 4.2: Add Hash Generator UI

Same pattern - update DevToolsPage.qml with hashToolComponent.

---

## Phase 5: Time Toolkit

### Task 5.1: Create Time Model

**Files:**
- Create: `crates/myme-ui/src/models/time_model.rs`

Uses chrono and chrono-tz for parsing, formatting, timezone conversion, and arithmetic.

### Task 5.2: Add Time Toolkit UI

Add timeToolComponent to DevToolsPage.qml.

---

## Phase 6: JSON Toolkit

### Task 6.1: Create JSON Model

**Files:**
- Create: `crates/myme-ui/src/models/json_model.rs`

Uses serde_json, serde_yaml, toml, jsonpath-rust.

### Task 6.2: Add JSON Toolkit UI (Tabbed)

Most complex UI with tabs for Format, Tree View, JSON Path, Convert, Diff.

---

## Final Phase: Integration & Testing

### Task 7.1: Manual Testing Checklist

- [ ] Encoding Hub: Base64, Hex, URL encode/decode
- [ ] UUID Generator: V1, V4, V5, V7 with all formats
- [ ] Hash Generator: Text hashing, file hashing, HMAC, verify
- [ ] Time Toolkit: Parse timestamps, timezones, arithmetic
- [ ] JSON Toolkit: Format, minify, tree view, convert, diff

### Task 7.2: Final Commit & PR

```bash
git push -u origin feature/devtools-expansion
gh pr create --title "feat(devtools): implement 5 developer tools" --body "..."
```
