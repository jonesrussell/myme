import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: devToolsPage
    title: "Developer Tools"

    // Current view: "index" or tool id
    property string currentTool: "index"
    property string searchQuery: ""

    // List of available tools
    property var tools: [
        {
            id: "jwt",
            name: "JWT Generator",
            description: "Generate and verify JSON Web Tokens for testing and development",
            icon: Icons.key,
            category: "Security"
        },
        {
            id: "encoding",
            name: "Encoding Hub",
            description: "Encode and decode Base64, Hex, and URL strings",
            icon: Icons.code,
            category: "Encoding"
        },
        {
            id: "uuid",
            name: "UUID Generator",
            description: "Generate UUIDs v1, v4, v5, v7 with format options",
            icon: Icons.squaresFour,
            category: "Generators"
        },
        {
            id: "json",
            name: "JSON Toolkit",
            description: "Format, validate, minify, convert to YAML/TOML, and query with JSONPath",
            icon: Icons.terminalWindow,
            category: "Formatting"
        },
        {
            id: "hash",
            name: "Hash Generator",
            description: "Generate MD5, SHA-1, SHA-256, SHA-512 hashes with HMAC support",
            icon: Icons.lock,
            category: "Security"
        },
        {
            id: "timestamp",
            name: "Time Toolkit",
            description: "Parse timestamps, convert timezones, and perform date arithmetic",
            icon: Icons.arrowsClockwise,
            category: "Conversion"
        },
        {
            id: "chunker",
            name: "Text Chunker",
            description: "Split large text into copyable chunks for pasting into AI tools with character limits",
            icon: Icons.scissors,
            category: "Text"
        }
    ]

    // Filter tools based on search query
    function filteredTools() {
        if (searchQuery.length === 0) {
            return tools;
        }
        const query = searchQuery.toLowerCase();
        return tools.filter(tool =>
            tool.name.toLowerCase().indexOf(query) !== -1 ||
            tool.description.toLowerCase().indexOf(query) !== -1 ||
            tool.category.toLowerCase().indexOf(query) !== -1
        );
    }

    background: Rectangle {
        color: Theme.background
    }

    // Instantiate the JwtModel from Rust (used when showing JWT tool)
    JwtModel {
        id: jwtModel
        payload: '{\n  "sub": "user123",\n  "name": "John Doe",\n  "iat": 1234567890\n}'
        algorithm: "HS256"
    }

    // Instantiate the EncodingModel from Rust
    EncodingModel {
        id: encodingModel
        encoding_type: "base64"
    }

    // Instantiate the UuidModel from Rust
    UuidModel {
        id: uuidModel
    }

    // Instantiate the HashModel from Rust
    HashModel {
        id: hashModel
    }

    // Instantiate the TimeModel from Rust
    TimeModel {
        id: timeModel
    }

    // Instantiate the JsonModel from Rust
    JsonModel {
        id: jsonModel
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            // Back button (when viewing a tool)
            ToolButton {
                visible: currentTool !== "index"
                text: Icons.caretLeft
                font.family: Icons.family
                font.pixelSize: 18
                onClicked: currentTool = "index"
                ToolTip.text: "Back to tools"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: Theme.text
                    font.pixelSize: 18
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }

            Label {
                text: {
                    if (currentTool === "index") return "Developer Tools";
                    for (let i = 0; i < tools.length; i++) {
                        if (tools[i].id === currentTool) return tools[i].name;
                    }
                    return "Tool";
                }
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: currentTool === "index" ? Theme.spacingMd : 0
            }
        }
    }

    // Main content - switches between index and tool views
    StackLayout {
        anchors.fill: parent
        currentIndex: currentTool === "index" ? 0 : 1

        // Index view with searchable cards
        ScrollView {
            id: devToolsScroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: devToolsScroll.viewport.width

            ColumnLayout {
                width: devToolsScroll.viewport.width
                spacing: Theme.spacingLg

                // Search bar
                Rectangle {
                    Layout.fillWidth: true
                    Layout.margins: Theme.spacingLg
                    Layout.bottomMargin: 0
                    height: 44
                    color: Theme.inputBg
                    border.color: searchField.activeFocus ? Theme.primary : Theme.inputBorder
                    border.width: searchField.activeFocus ? 2 : 1
                    radius: Theme.inputRadius

                    Behavior on border.color {
                        ColorAnimation {
                            duration: 100
                        }
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingSm
                        spacing: Theme.spacingSm

                        Label {
                            text: Icons.list
                            font.family: Icons.family
                            font.pixelSize: 18
                            color: Theme.textMuted
                        }

                        TextField {
                            id: searchField
                            Layout.fillWidth: true
                            placeholderText: "Search tools..."
                            color: Theme.text
                            placeholderTextColor: Theme.textMuted
                            onTextChanged: searchQuery = text

                            background: Rectangle {
                                color: "transparent"
                            }
                        }

                        Label {
                            visible: searchQuery.length > 0
                            text: Icons.x
                            font.family: Icons.family
                            font.pixelSize: 14
                            color: Theme.textSecondary

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    searchField.text = "";
                                    searchQuery = "";
                                }
                            }
                        }
                    }
                }

                // Tools count
                Label {
                    Layout.leftMargin: Theme.spacingLg
                    text: filteredTools().length + " tool" + (filteredTools().length !== 1 ? "s" : "") + " available"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                }

                // Tools grid
                GridLayout {
                    Layout.fillWidth: true
                    Layout.margins: Theme.spacingLg
                    Layout.topMargin: 0
                    columns: Responsive.columnsFor(devToolsPage.width - Theme.spacingLg * 2, 280, 4)
                    columnSpacing: Theme.spacingMd
                    rowSpacing: Theme.spacingMd

                    Repeater {
                        model: filteredTools()

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredWidth: 260
                            Layout.preferredHeight: 140
                            color: modelData.comingSoon ? Theme.surfaceAlt : (toolCardMouse.containsMouse ? Theme.surfaceHover : Theme.surface)
                            border.color: toolCardMouse.containsMouse && !modelData.comingSoon ? Theme.primary : (Theme.isDark ? "#ffffff08" : "#00000008")
                            border.width: 1
                            radius: Theme.cardRadius
                            opacity: modelData.comingSoon ? 0.7 : 1.0

                            Behavior on color {
                                ColorAnimation {
                                    duration: 100
                                }
                            }

                            Behavior on border.color {
                                ColorAnimation {
                                    duration: 100
                                }
                            }

                            MouseArea {
                                id: toolCardMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: modelData.comingSoon ? Qt.ForbiddenCursor : Qt.PointingHandCursor
                                onClicked: {
                                    if (!modelData.comingSoon) {
                                        currentTool = modelData.id;
                                    }
                                }
                            }

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingMd
                                spacing: Theme.spacingSm

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: Theme.spacingSm

                                    Rectangle {
                                        width: 40
                                        height: 40
                                        radius: 8
                                        color: modelData.comingSoon ? Theme.textMuted + "20" : Theme.primary + "20"

                                        Label {
                                            anchors.centerIn: parent
                                            text: modelData.icon
                                            font.family: Icons.family
                                            font.pixelSize: 20
                                            color: modelData.comingSoon ? Theme.textMuted : Theme.primary
                                        }
                                    }

                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2

                                        Label {
                                            text: modelData.name
                                            font.pixelSize: Theme.fontSizeNormal
                                            font.bold: true
                                            color: Theme.text
                                        }

                                        Rectangle {
                                            width: categoryLabel.implicitWidth + Theme.spacingSm
                                            height: categoryLabel.implicitHeight + 4
                                            radius: 4
                                            color: modelData.comingSoon ? Theme.warning + "20" : Theme.info + "20"

                                            Label {
                                                id: categoryLabel
                                                anchors.centerIn: parent
                                                text: modelData.comingSoon ? "Coming Soon" : modelData.category
                                                font.pixelSize: Theme.fontSizeSmall - 2
                                                color: modelData.comingSoon ? Theme.warning : Theme.info
                                            }
                                        }
                                    }
                                }

                                Label {
                                    text: modelData.description
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                    wrapMode: Text.WordWrap
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                }
                            }
                        }
                    }
                }

                // Empty state
                ColumnLayout {
                    visible: filteredTools().length === 0
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: Theme.spacingLg * 2
                    spacing: Theme.spacingMd

                    Label {
                        text: Icons.wrench
                        font.family: Icons.family
                        font.pixelSize: 48
                        color: Theme.textMuted
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "No tools found"
                        font.pixelSize: Theme.fontSizeLarge
                        font.bold: true
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Try a different search term"
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        Layout.alignment: Qt.AlignHCenter
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }

        // Tool detail view
        Loader {
            id: toolLoader
            Layout.fillWidth: true
            Layout.fillHeight: true
            active: currentTool !== "index"
            sourceComponent: {
                if (currentTool === "jwt") return jwtToolComponent;
                if (currentTool === "encoding") return encodingToolComponent;
                if (currentTool === "uuid") return uuidToolComponent;
                if (currentTool === "hash") return hashToolComponent;
                if (currentTool === "timestamp") return timeToolComponent;
                if (currentTool === "json") return jsonToolComponent;
                if (currentTool === "chunker") return chunkerToolComponent;
                return null;
            }
        }
    }

    // JWT Tool Component
    Component {
        id: jwtToolComponent

        ScrollView {
            anchors.fill: parent
            anchors.margins: Theme.spacingLg
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingLg

                // JWT Generator Section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: jwtContent.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    border.width: 1
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: jwtContent
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        // Error message banner
                        Rectangle {
                            visible: jwtModel.error_message.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 50
                            color: Theme.errorBg
                            border.color: "transparent"
                            border.width: 0
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
                                    text: jwtModel.error_message
                                    color: Theme.error
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                }
                            }
                        }

                        // Payload input
                        Label {
                            text: "Payload (JSON)"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: true
                            color: Theme.text
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 150
                            color: Theme.inputBg
                            border.color: payloadField.activeFocus ? Theme.primary : Theme.inputBorder
                            border.width: payloadField.activeFocus ? 2 : 1
                            radius: Theme.inputRadius

                            Behavior on border.color {
                                ColorAnimation {
                                    duration: 100
                                }
                            }

                            ScrollView {
                                anchors.fill: parent
                                anchors.margins: 2

                                TextArea {
                                    id: payloadField
                                    text: jwtModel.payload
                                    placeholderText: '{\n  "sub": "user123",\n  "name": "John Doe",\n  "iat": 1234567890\n}'
                                    wrapMode: TextEdit.Wrap
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    onTextChanged: jwtModel.payload = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }
                        }

                        // Secret key input
                        Label {
                            text: "Secret Key"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            Rectangle {
                                Layout.fillWidth: true
                                height: 40
                                color: Theme.inputBg
                                border.color: secretField.activeFocus ? Theme.primary : Theme.inputBorder
                                border.width: secretField.activeFocus ? 2 : 1
                                radius: Theme.inputRadius

                                Behavior on border.color {
                                    ColorAnimation {
                                        duration: 100
                                    }
                                }

                                TextField {
                                    id: secretField
                                    anchors.fill: parent
                                    anchors.margins: 2
                                    placeholderText: "Enter your secret key..."
                                    echoMode: showSecretCheckbox.checked ? TextInput.Normal : TextInput.Password
                                    text: jwtModel.secret
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    onTextChanged: jwtModel.secret = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }

                            CheckBox {
                                id: showSecretCheckbox
                                text: "Show"

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.text
                                    leftPadding: parent.indicator.width + parent.spacing
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }
                        }

                        // Algorithm selector
                        Label {
                            text: "Algorithm"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: true
                            color: Theme.text
                        }

                        ComboBox {
                            id: algorithmCombo
                            Layout.preferredWidth: 200
                            model: ["HS256", "HS384", "HS512"]
                            currentIndex: {
                                const alg = jwtModel.algorithm;
                                if (alg === "HS384")
                                    return 1;
                                if (alg === "HS512")
                                    return 2;
                                return 0;
                            }
                            onCurrentTextChanged: jwtModel.algorithm = currentText
                        }

                        // Generate button
                        Button {
                            text: "Generate Token"
                            Layout.preferredWidth: 180
                            Layout.preferredHeight: 44
                            onClicked: jwtModel.generate_token()

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
                                verticalAlignment: Text.AlignVCenter
                            }
                        }

                        // Generated token output
                        Label {
                            text: "Generated Token"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: true
                            color: Theme.text
                            visible: jwtModel.generated_token.length > 0
                        }

                        Rectangle {
                            visible: jwtModel.generated_token.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120
                            color: Theme.successBg
                            border.color: Theme.success
                            border.width: 2
                            radius: Theme.cardRadius

                            ScrollView {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingMd

                                TextArea {
                                    id: tokenOutput
                                    text: jwtModel.generated_token
                                    readOnly: true
                                    wrapMode: TextEdit.WrapAnywhere
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                    selectByMouse: true

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }
                        }

                        // Copy button
                        RowLayout {
                            visible: jwtModel.generated_token.length > 0
                            spacing: Theme.spacingMd

                            Button {
                                text: "Copy to Clipboard"
                                onClicked: {
                                    tokenOutput.selectAll();
                                    tokenOutput.copy();
                                    tokenOutput.deselect();
                                    copyFeedback.visible = true;
                                    copyFeedbackTimer.start();
                                }

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                                    border.color: Theme.border
                                    border.width: 1
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.text
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Rectangle {
                                visible: copyFeedback.visible
                                width: copyFeedback.implicitWidth + Theme.spacingMd
                                height: copyFeedback.implicitHeight + Theme.spacingXs
                                radius: 4
                                color: Theme.success + "20"

                                Label {
                                    id: copyFeedback
                                    anchors.centerIn: parent
                                    text: Icons.check + " Copied!"
                                    font.family: Icons.family
                                    color: Theme.success
                                    font.bold: true
                                    visible: false

                                    Timer {
                                        id: copyFeedbackTimer
                                        interval: 2000
                                        onTriggered: copyFeedback.visible = false
                                    }
                                }
                            }
                        }

                        // Token info
                        Rectangle {
                            visible: jwtModel.generated_token.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: infoColumn.height + Theme.spacingMd * 2
                            color: Theme.infoBg
                            border.color: Theme.info
                            border.width: 1
                            radius: Theme.cardRadius

                            ColumnLayout {
                                id: infoColumn
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.top: parent.top
                                anchors.margins: Theme.spacingMd
                                spacing: Theme.spacingXs

                                Label {
                                    text: "Token Information"
                                    font.bold: true
                                    font.pixelSize: Theme.fontSizeNormal
                                    color: Theme.info
                                }

                                Label {
                                    text: "Algorithm: " + jwtModel.algorithm
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }

                                Label {
                                    text: "Token Length: " + jwtModel.generated_token.length + " characters"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }

                                Label {
                                    text: "Parts: " + jwtModel.generated_token.split(".").length + " (Header.Payload.Signature)"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                            }
                        }
                    }
                }

                // Spacer
                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }

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
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
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
                    border.color: "transparent"
                    border.width: 0
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
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
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
                    border.color: encodingModel.output.length > 0 ? Theme.success : (Theme.isDark ? "#ffffff08" : "#00000008")
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

    // UUID Generator Component
    Component {
        id: uuidToolComponent

        ScrollView {
            anchors.fill: parent
            anchors.margins: Theme.spacingLg
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingLg

                // Settings card
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: uuidSettingsColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: uuidSettingsColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        // UUID Version selector
                        Label {
                            text: "UUID Version"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            spacing: Theme.spacingSm

                            Repeater {
                                model: [
                                    { id: "v1", label: "v1 (Time)", desc: "Time-based" },
                                    { id: "v4", label: "v4 (Random)", desc: "Random" },
                                    { id: "v5", label: "v5 (Name)", desc: "Namespace + Name" },
                                    { id: "v7", label: "v7 (Time)", desc: "Unix timestamp" }
                                ]

                                Button {
                                    text: modelData.label
                                    checkable: true
                                    checked: uuidModel.version === modelData.id
                                    onClicked: uuidModel.version = modelData.id

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

                                    ToolTip.text: modelData.desc
                                    ToolTip.visible: hovered
                                }
                            }
                        }

                        // Format selector
                        Label {
                            text: "Format"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            spacing: Theme.spacingSm

                            Repeater {
                                model: [
                                    { id: "standard", label: "Standard", example: "550e8400-e29b-41d4-a716-446655440000" },
                                    { id: "uppercase", label: "Uppercase", example: "550E8400-E29B-41D4-A716-446655440000" },
                                    { id: "no-dashes", label: "No Dashes", example: "550e8400e29b41d4a716446655440000" },
                                    { id: "braces", label: "Braces", example: "{550e8400-e29b-41d4-a716-446655440000}" }
                                ]

                                Button {
                                    text: modelData.label
                                    checkable: true
                                    checked: uuidModel.format === modelData.id
                                    onClicked: uuidModel.format = modelData.id

                                    background: Rectangle {
                                        radius: Theme.buttonRadius
                                        color: parent.checked ? Theme.info : (parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt)
                                    }

                                    contentItem: Text {
                                        text: parent.text
                                        color: parent.checked ? Theme.primaryText : Theme.text
                                        font.pixelSize: Theme.fontSizeSmall
                                        horizontalAlignment: Text.AlignHCenter
                                        verticalAlignment: Text.AlignVCenter
                                    }

                                    ToolTip.text: modelData.example
                                    ToolTip.visible: hovered
                                }
                            }
                        }

                        // Count selector
                        RowLayout {
                            spacing: Theme.spacingMd

                            Label {
                                text: "Count:"
                                font.bold: true
                                color: Theme.text
                            }

                            SpinBox {
                                id: uuidCountSpinner
                                from: 1
                                to: 100
                                value: uuidModel.count
                                onValueChanged: uuidModel.count = value
                                editable: true
                            }
                        }

                        // V5-specific options
                        ColumnLayout {
                            visible: uuidModel.version === "v5"
                            Layout.fillWidth: true
                            spacing: Theme.spacingSm

                            Label {
                                text: "Namespace"
                                font.bold: true
                                color: Theme.text
                            }

                            RowLayout {
                                spacing: Theme.spacingSm

                                Repeater {
                                    model: [
                                        { id: "dns", label: "DNS" },
                                        { id: "url", label: "URL" },
                                        { id: "oid", label: "OID" },
                                        { id: "x500", label: "X500" },
                                        { id: "custom", label: "Custom" }
                                    ]

                                    Button {
                                        text: modelData.label
                                        checkable: true
                                        checked: uuidModel.namespace_type === modelData.id
                                        onClicked: uuidModel.namespace_type = modelData.id

                                        background: Rectangle {
                                            radius: Theme.buttonRadius
                                            color: parent.checked ? Theme.warning : (parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt)
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

                            // Custom namespace input
                            Rectangle {
                                visible: uuidModel.namespace_type === "custom"
                                Layout.fillWidth: true
                                height: 40
                                color: Theme.inputBg
                                border.color: Theme.inputBorder
                                radius: Theme.inputRadius

                                TextField {
                                    anchors.fill: parent
                                    anchors.margins: 2
                                    placeholderText: "Custom namespace UUID..."
                                    text: uuidModel.custom_namespace
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    font.family: "Consolas, Monaco, monospace"
                                    onTextChanged: uuidModel.custom_namespace = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }

                            Label {
                                text: "Name"
                                font.bold: true
                                color: Theme.text
                            }

                            Rectangle {
                                Layout.fillWidth: true
                                height: 40
                                color: Theme.inputBg
                                border.color: Theme.inputBorder
                                radius: Theme.inputRadius

                                TextField {
                                    anchors.fill: parent
                                    anchors.margins: 2
                                    placeholderText: "Enter name to hash..."
                                    text: uuidModel.name
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    onTextChanged: uuidModel.name = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }
                        }
                    }
                }

                // Error banner
                Rectangle {
                    visible: uuidModel.error_message.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 50
                    color: Theme.errorBg
                    border.color: "transparent"
                    border.width: 0
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
                            text: uuidModel.error_message
                            color: Theme.error
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                // Generate button
                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: Theme.spacingMd

                    Button {
                        text: Icons.squaresFour + " Generate"
                        font.family: Icons.family
                        onClicked: uuidModel.generate()

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
                        text: "Clear"
                        onClicked: uuidModel.clear()

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

                // Generated UUIDs
                Rectangle {
                    visible: uuidModel.generated_uuids.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.min(400, uuidListColumn.implicitHeight + Theme.spacingLg * 2)
                    color: Theme.surface
                    border.color: Theme.success
                    border.width: 2
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: uuidListColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingSm

                        RowLayout {
                            Layout.fillWidth: true

                            Label {
                                text: "Generated UUIDs (" + uuidModel.generated_uuids.length + ")"
                                font.bold: true
                                color: Theme.text
                                Layout.fillWidth: true
                            }

                            Button {
                                text: "Copy All"
                                onClicked: {
                                    let allUuids = [];
                                    for (let i = 0; i < uuidModel.generated_uuids.length; i++) {
                                        allUuids.push(uuidModel.generated_uuids[i]);
                                    }
                                    uuidOutputArea.text = allUuids.join("\n");
                                    uuidOutputArea.selectAll();
                                    uuidOutputArea.copy();
                                    uuidOutputArea.deselect();
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
                            clip: true

                            TextArea {
                                id: uuidOutputArea
                                text: {
                                    let result = [];
                                    for (let i = 0; i < uuidModel.generated_uuids.length; i++) {
                                        result.push(uuidModel.generated_uuids[i]);
                                    }
                                    return result.join("\n");
                                }
                                readOnly: true
                                wrapMode: TextEdit.NoWrap
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

    // Hash Generator Component
    Component {
        id: hashToolComponent

        ScrollView {
            anchors.fill: parent
            anchors.margins: Theme.spacingLg
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingLg

                // Input section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: hashInputColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: hashInputColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Input Text"
                            font.bold: true
                            color: Theme.text
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120
                            color: Theme.inputBg
                            border.color: hashInputField.activeFocus ? Theme.primary : Theme.inputBorder
                            radius: Theme.inputRadius

                            ScrollView {
                                anchors.fill: parent
                                anchors.margins: 2

                                TextArea {
                                    id: hashInputField
                                    text: hashModel.input
                                    placeholderText: "Enter text to hash..."
                                    wrapMode: TextEdit.Wrap
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    onTextChanged: hashModel.input = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }
                        }

                        RowLayout {
                            spacing: Theme.spacingMd

                            Button {
                                text: Icons.lock + " Hash Text"
                                font.family: Icons.family
                                onClicked: hashModel.hash_text()

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
                                text: "Clear"
                                onClicked: {
                                    hashModel.clear()
                                    hashInputField.text = ""
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
                    }
                }

                // Error banner
                Rectangle {
                    visible: hashModel.error_message.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 50
                    color: Theme.errorBg
                    border.color: "transparent"
                    border.width: 0
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
                            text: hashModel.error_message
                            color: Theme.error
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                // Hash results
                Rectangle {
                    visible: hashModel.hash_md5.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: hashResultsColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.success
                    border.width: 2
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: hashResultsColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Hash Results"
                            font.bold: true
                            color: Theme.text
                        }

                        // MD5
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            RowLayout {
                                Label {
                                    text: "MD5"
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.preferredWidth: 80
                                }

                                Button {
                                    text: "Copy"
                                    onClicked: {
                                        md5Output.selectAll()
                                        md5Output.copy()
                                        md5Output.deselect()
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

                            TextField {
                                id: md5Output
                                Layout.fillWidth: true
                                text: hashModel.hash_md5
                                readOnly: true
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

                        // SHA-1
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            RowLayout {
                                Label {
                                    text: "SHA-1"
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.preferredWidth: 80
                                }

                                Button {
                                    text: "Copy"
                                    onClicked: {
                                        sha1Output.selectAll()
                                        sha1Output.copy()
                                        sha1Output.deselect()
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

                            TextField {
                                id: sha1Output
                                Layout.fillWidth: true
                                text: hashModel.hash_sha1
                                readOnly: true
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

                        // SHA-256
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            RowLayout {
                                Label {
                                    text: "SHA-256"
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.preferredWidth: 80
                                }

                                Button {
                                    text: "Copy"
                                    onClicked: {
                                        sha256Output.selectAll()
                                        sha256Output.copy()
                                        sha256Output.deselect()
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

                            TextField {
                                id: sha256Output
                                Layout.fillWidth: true
                                text: hashModel.hash_sha256
                                readOnly: true
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

                        // SHA-512
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            RowLayout {
                                Label {
                                    text: "SHA-512"
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.preferredWidth: 80
                                }

                                Button {
                                    text: "Copy"
                                    onClicked: {
                                        sha512Output.selectAll()
                                        sha512Output.copy()
                                        sha512Output.deselect()
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

                            TextArea {
                                id: sha512Output
                                Layout.fillWidth: true
                                Layout.preferredHeight: 60
                                text: hashModel.hash_sha512
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

                // HMAC section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: hmacColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: hmacColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        RowLayout {
                            Label {
                                text: "HMAC"
                                font.bold: true
                                color: Theme.text
                            }

                            CheckBox {
                                id: hmacEnabledCheck
                                checked: hashModel.hmac_enabled
                                onCheckedChanged: hashModel.hmac_enabled = checked
                            }
                        }

                        ColumnLayout {
                            visible: hashModel.hmac_enabled
                            Layout.fillWidth: true
                            spacing: Theme.spacingSm

                            Label {
                                text: "Secret Key"
                                color: Theme.textSecondary
                            }

                            TextField {
                                Layout.fillWidth: true
                                text: hashModel.hmac_key
                                placeholderText: "Enter HMAC secret key..."
                                color: Theme.text
                                placeholderTextColor: Theme.textMuted
                                onTextChanged: hashModel.hmac_key = text

                                background: Rectangle {
                                    color: Theme.inputBg
                                    border.color: Theme.inputBorder
                                    radius: Theme.inputRadius
                                }
                            }

                            RowLayout {
                                spacing: Theme.spacingSm

                                Label {
                                    text: "Algorithm:"
                                    color: Theme.textSecondary
                                }

                                Repeater {
                                    model: [
                                        { id: "sha256", label: "SHA-256" },
                                        { id: "sha512", label: "SHA-512" }
                                    ]

                                    Button {
                                        text: modelData.label
                                        checkable: true
                                        checked: hashModel.hmac_algorithm === modelData.id
                                        onClicked: hashModel.hmac_algorithm = modelData.id

                                        background: Rectangle {
                                            radius: Theme.buttonRadius
                                            color: parent.checked ? Theme.info : (parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt)
                                        }

                                        contentItem: Text {
                                            text: parent.text
                                            color: parent.checked ? Theme.primaryText : Theme.text
                                            font.pixelSize: Theme.fontSizeSmall
                                            horizontalAlignment: Text.AlignHCenter
                                        }
                                    }
                                }
                            }

                            Button {
                                text: "Compute HMAC"
                                onClicked: hashModel.compute_hmac()

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.primaryText
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }

                            ColumnLayout {
                                visible: hashModel.hmac_result.length > 0
                                Layout.fillWidth: true
                                spacing: 4

                                Label {
                                    text: "HMAC Result"
                                    font.bold: true
                                    color: Theme.success
                                }

                                TextArea {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 60
                                    text: hashModel.hmac_result
                                    readOnly: true
                                    wrapMode: TextEdit.WrapAnywhere
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                    selectByMouse: true

                                    background: Rectangle {
                                        color: Theme.successBg
                                        border.color: Theme.success
                                        radius: Theme.inputRadius
                                    }
                                }
                            }
                        }
                    }
                }

                // Compare section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: compareColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: compareColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Verify Hash"
                            font.bold: true
                            color: Theme.text
                        }

                        TextField {
                            id: compareHashField
                            Layout.fillWidth: true
                            text: hashModel.compare_hash
                            placeholderText: "Paste hash to verify..."
                            color: Theme.text
                            placeholderTextColor: Theme.textMuted
                            font.family: "Consolas, Monaco, monospace"
                            onTextChanged: hashModel.compare_hash = text

                            background: Rectangle {
                                color: Theme.inputBg
                                border.color: Theme.inputBorder
                                radius: Theme.inputRadius
                            }
                        }

                        Button {
                            text: "Compare"
                            onClicked: hashModel.compare()

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

                        Rectangle {
                            visible: hashModel.compare_result.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 50
                            color: hashModel.compare_result.startsWith("match") ? Theme.successBg : Theme.errorBg
                            border.color: hashModel.compare_result.startsWith("match") ? Theme.success : Theme.error
                            radius: Theme.cardRadius

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingMd
                                spacing: Theme.spacingMd

                                Label {
                                    text: hashModel.compare_result.startsWith("match") ? Icons.check : Icons.x
                                    font.family: Icons.family
                                    font.pixelSize: 24
                                    color: hashModel.compare_result.startsWith("match") ? Theme.success : Theme.error
                                }

                                Label {
                                    text: {
                                        if (hashModel.compare_result.startsWith("match:")) {
                                            return "Match! (" + hashModel.compare_result.split(":")[1] + ")";
                                        }
                                        return "No match found";
                                    }
                                    font.bold: true
                                    color: hashModel.compare_result.startsWith("match") ? Theme.success : Theme.error
                                }
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }

    // Time Toolkit Component
    Component {
        id: timeToolComponent

        ScrollView {
            anchors.fill: parent
            anchors.margins: Theme.spacingLg
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingLg

                // Current time button
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: currentTimeColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: currentTimeColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        RowLayout {
                            spacing: Theme.spacingMd

                            Button {
                                text: Icons.arrowsClockwise + " Get Current Time"
                                font.family: Icons.family
                                onClicked: timeModel.get_current_time()

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
                                text: "Clear All"
                                onClicked: timeModel.clear()

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

                        TextArea {
                            visible: timeModel.current_time.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 80
                            text: timeModel.current_time
                            readOnly: true
                            font.family: "Consolas, Monaco, monospace"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.text
                            selectByMouse: true

                            background: Rectangle {
                                color: Theme.infoBg
                                border.color: Theme.info
                                radius: Theme.inputRadius
                            }
                        }
                    }
                }

                // Error banner
                Rectangle {
                    visible: timeModel.error_message.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 50
                    color: Theme.errorBg
                    border.color: "transparent"
                    border.width: 0
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
                            text: timeModel.error_message
                            color: Theme.error
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                // Parse timestamp section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: timestampColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: timestampColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Parse Unix Timestamp"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            TextField {
                                id: timestampInput
                                Layout.fillWidth: true
                                text: timeModel.input_timestamp
                                placeholderText: "Enter Unix timestamp (seconds or milliseconds)..."
                                color: Theme.text
                                placeholderTextColor: Theme.textMuted
                                font.family: "Consolas, Monaco, monospace"
                                onTextChanged: timeModel.input_timestamp = text

                                background: Rectangle {
                                    color: Theme.inputBg
                                    border.color: Theme.inputBorder
                                    radius: Theme.inputRadius
                                }
                            }

                            Button {
                                text: "Parse"
                                onClicked: timeModel.parse_timestamp()

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.primaryText
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }
                    }
                }

                // Parse datetime section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: datetimeColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: datetimeColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Parse Date/Time String"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            TextField {
                                id: datetimeInput
                                Layout.fillWidth: true
                                text: timeModel.input_datetime
                                placeholderText: "YYYY-MM-DD HH:MM:SS"
                                color: Theme.text
                                placeholderTextColor: Theme.textMuted
                                font.family: "Consolas, Monaco, monospace"
                                onTextChanged: timeModel.input_datetime = text

                                background: Rectangle {
                                    color: Theme.inputBg
                                    border.color: Theme.inputBorder
                                    radius: Theme.inputRadius
                                }
                            }

                            ComboBox {
                                id: inputTimezoneCombo
                                model: ["UTC", "Local", "America/New_York", "America/Los_Angeles", "Europe/London", "Europe/Paris", "Asia/Tokyo"]
                                currentIndex: 0
                                onCurrentTextChanged: timeModel.input_timezone = currentText
                            }

                            Button {
                                text: "Parse"
                                onClicked: timeModel.parse_datetime()

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.primaryText
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }
                    }
                }

                // Results section
                Rectangle {
                    visible: timeModel.output_timestamp.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: resultsColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.success
                    border.width: 2
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: resultsColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Parsed Results"
                            font.bold: true
                            color: Theme.text
                        }

                        GridLayout {
                            Layout.fillWidth: true
                            columns: 2
                            columnSpacing: Theme.spacingMd
                            rowSpacing: Theme.spacingSm

                            Label {
                                text: "Timestamp:"
                                font.bold: true
                                color: Theme.textSecondary
                            }
                            TextField {
                                Layout.fillWidth: true
                                text: timeModel.output_timestamp
                                readOnly: true
                                font.family: "Consolas, Monaco, monospace"
                                color: Theme.text
                                selectByMouse: true
                                background: Rectangle { color: Theme.inputBg; radius: Theme.inputRadius }
                            }

                            Label {
                                text: "ISO 8601:"
                                font.bold: true
                                color: Theme.textSecondary
                            }
                            TextField {
                                Layout.fillWidth: true
                                text: timeModel.output_iso8601
                                readOnly: true
                                font.family: "Consolas, Monaco, monospace"
                                color: Theme.text
                                selectByMouse: true
                                background: Rectangle { color: Theme.inputBg; radius: Theme.inputRadius }
                            }

                            Label {
                                text: "RFC 2822:"
                                font.bold: true
                                color: Theme.textSecondary
                            }
                            TextField {
                                Layout.fillWidth: true
                                text: timeModel.output_rfc2822
                                readOnly: true
                                font.family: "Consolas, Monaco, monospace"
                                color: Theme.text
                                selectByMouse: true
                                background: Rectangle { color: Theme.inputBg; radius: Theme.inputRadius }
                            }

                            Label {
                                text: "Local:"
                                font.bold: true
                                color: Theme.textSecondary
                            }
                            TextField {
                                Layout.fillWidth: true
                                text: timeModel.output_local
                                readOnly: true
                                font.family: "Consolas, Monaco, monospace"
                                color: Theme.text
                                selectByMouse: true
                                background: Rectangle { color: Theme.inputBg; radius: Theme.inputRadius }
                            }

                            Label {
                                text: "Relative:"
                                font.bold: true
                                color: Theme.textSecondary
                            }
                            Label {
                                text: timeModel.output_relative
                                color: Theme.info
                                font.bold: true
                            }
                        }
                    }
                }

                // Timezone conversion
                Rectangle {
                    visible: timeModel.output_timestamp.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: tzColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: tzColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Convert to Timezone"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            spacing: Theme.spacingMd

                            ComboBox {
                                id: targetTimezoneCombo
                                model: ["Local", "UTC", "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles", "Europe/London", "Europe/Paris", "Europe/Berlin", "Asia/Tokyo", "Asia/Shanghai", "Australia/Sydney"]
                                currentIndex: 0
                                onCurrentTextChanged: timeModel.target_timezone = currentText
                            }

                            Button {
                                text: "Convert"
                                onClicked: timeModel.convert_timezone()

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
                        }

                        TextField {
                            visible: timeModel.converted_time.length > 0
                            Layout.fillWidth: true
                            text: timeModel.converted_time
                            readOnly: true
                            font.family: "Consolas, Monaco, monospace"
                            color: Theme.text
                            selectByMouse: true

                            background: Rectangle {
                                color: Theme.successBg
                                border.color: Theme.success
                                radius: Theme.inputRadius
                            }
                        }
                    }
                }

                // Date arithmetic
                Rectangle {
                    visible: timeModel.output_timestamp.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: arithmeticColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: arithmeticColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Date Arithmetic"
                            font.bold: true
                            color: Theme.text
                        }

                        GridLayout {
                            columns: 4
                            columnSpacing: Theme.spacingMd
                            rowSpacing: Theme.spacingSm

                            Label { text: "Days:"; color: Theme.textSecondary }
                            SpinBox {
                                from: -365
                                to: 365
                                value: timeModel.add_days
                                onValueChanged: timeModel.add_days = value
                                editable: true
                            }

                            Label { text: "Hours:"; color: Theme.textSecondary }
                            SpinBox {
                                from: -24
                                to: 24
                                value: timeModel.add_hours
                                onValueChanged: timeModel.add_hours = value
                                editable: true
                            }

                            Label { text: "Minutes:"; color: Theme.textSecondary }
                            SpinBox {
                                from: -60
                                to: 60
                                value: timeModel.add_minutes
                                onValueChanged: timeModel.add_minutes = value
                                editable: true
                            }

                            Label { text: "Seconds:"; color: Theme.textSecondary }
                            SpinBox {
                                from: -60
                                to: 60
                                value: timeModel.add_seconds
                                onValueChanged: timeModel.add_seconds = value
                                editable: true
                            }
                        }

                        Button {
                            text: "Calculate"
                            onClicked: timeModel.apply_arithmetic()

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

                        TextArea {
                            visible: timeModel.arithmetic_result.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 80
                            text: timeModel.arithmetic_result
                            readOnly: true
                            font.family: "Consolas, Monaco, monospace"
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.text
                            selectByMouse: true

                            background: Rectangle {
                                color: Theme.successBg
                                border.color: Theme.success
                                radius: Theme.inputRadius
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }

    // JSON Toolkit Component
    Component {
        id: jsonToolComponent

        ScrollView {
            anchors.fill: parent
            anchors.margins: Theme.spacingLg
            clip: true

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingLg

                // Input section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 250
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        RowLayout {
                            Layout.fillWidth: true

                            Label {
                                text: "JSON Input"
                                font.bold: true
                                color: Theme.text
                                Layout.fillWidth: true
                            }

                            Button {
                                text: "Clear"
                                onClicked: {
                                    jsonModel.clear()
                                    jsonInputField.text = ""
                                }

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.textSecondary
                                    font.pixelSize: Theme.fontSizeSmall
                                }
                            }
                        }

                        ScrollView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            TextArea {
                                id: jsonInputField
                                text: jsonModel.input
                                placeholderText: '{\n  "key": "value"\n}'
                                wrapMode: TextEdit.Wrap
                                font.family: "Consolas, Monaco, monospace"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.text
                                placeholderTextColor: Theme.textMuted
                                onTextChanged: jsonModel.input = text

                                background: Rectangle {
                                    color: Theme.inputBg
                                    radius: Theme.inputRadius
                                }
                            }
                        }
                    }
                }

                // Error banner
                Rectangle {
                    visible: jsonModel.error_message.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 50
                    color: Theme.errorBg
                    border.color: "transparent"
                    border.width: 0
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
                            text: jsonModel.error_message
                            color: Theme.error
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                // Action buttons
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: actionButtonsRow.implicitHeight + Theme.spacingMd * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    RowLayout {
                        id: actionButtonsRow
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingMd

                        Button {
                            text: "Format"
                            onClicked: jsonModel.format_json()

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
                            text: "Minify"
                            onClicked: jsonModel.minify_json()

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
                            text: "Validate"
                            onClicked: jsonModel.validate_json()

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

                        Item { Layout.fillWidth: true }

                        // Validation indicator
                        Rectangle {
                            visible: jsonModel.validation_message.length > 0
                            Layout.preferredWidth: validationLabel.implicitWidth + 20
                            Layout.preferredHeight: 30
                            radius: Theme.buttonRadius
                            color: jsonModel.is_valid ? Theme.successBg : Theme.errorBg
                            border.color: jsonModel.is_valid ? Theme.success : Theme.error

                            Label {
                                id: validationLabel
                                anchors.centerIn: parent
                                text: jsonModel.is_valid ? Icons.check + " Valid" : Icons.x + " Invalid"
                                font.family: Icons.family
                                color: jsonModel.is_valid ? Theme.success : Theme.error
                                font.pixelSize: Theme.fontSizeSmall
                            }
                        }
                    }
                }

                // Output section
                Rectangle {
                    visible: jsonModel.output.length > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 200
                    color: Theme.surface
                    border.color: Theme.success
                    border.width: 2
                    radius: Theme.cardRadius

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
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
                                text: "Copy"
                                onClicked: {
                                    jsonOutputField.selectAll()
                                    jsonOutputField.copy()
                                    jsonOutputField.deselect()
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
                                id: jsonOutputField
                                text: jsonModel.output
                                readOnly: true
                                wrapMode: TextEdit.Wrap
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

                // JSONPath Query section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: jsonpathColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: jsonpathColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "JSONPath Query"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            TextField {
                                Layout.fillWidth: true
                                text: jsonModel.jsonpath_query
                                placeholderText: "$ (root), $.key, $..items[*].name"
                                color: Theme.text
                                placeholderTextColor: Theme.textMuted
                                font.family: "Consolas, Monaco, monospace"
                                onTextChanged: jsonModel.jsonpath_query = text

                                background: Rectangle {
                                    color: Theme.inputBg
                                    border.color: Theme.inputBorder
                                    radius: Theme.inputRadius
                                }
                            }

                            Button {
                                text: "Query"
                                onClicked: jsonModel.query_jsonpath()

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.primaryText
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }

                        ScrollView {
                            visible: jsonModel.jsonpath_result.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 100

                            TextArea {
                                text: jsonModel.jsonpath_result
                                readOnly: true
                                wrapMode: TextEdit.Wrap
                                font.family: "Consolas, Monaco, monospace"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.text
                                selectByMouse: true

                                background: Rectangle {
                                    color: Theme.infoBg
                                    border.color: Theme.info
                                    radius: Theme.inputRadius
                                }
                            }
                        }
                    }
                }

                // Convert to format section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: convertColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: convertColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Convert to Format"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            spacing: Theme.spacingMd

                            Repeater {
                                model: [
                                    { id: "yaml", label: "YAML" },
                                    { id: "toml", label: "TOML" }
                                ]

                                Button {
                                    text: modelData.label
                                    checkable: true
                                    checked: jsonModel.convert_format === modelData.id
                                    onClicked: jsonModel.convert_format = modelData.id

                                    background: Rectangle {
                                        radius: Theme.buttonRadius
                                        color: parent.checked ? Theme.info : (parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt)
                                    }

                                    contentItem: Text {
                                        text: parent.text
                                        color: parent.checked ? Theme.primaryText : Theme.text
                                        font.pixelSize: Theme.fontSizeSmall
                                        horizontalAlignment: Text.AlignHCenter
                                    }
                                }
                            }

                            Button {
                                text: "Convert"
                                onClicked: jsonModel.convert_to_format()

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.primaryText
                                    font.pixelSize: Theme.fontSizeNormal
                                    horizontalAlignment: Text.AlignHCenter
                                }
                            }
                        }

                        ScrollView {
                            visible: jsonModel.converted_output.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 150

                            TextArea {
                                text: jsonModel.converted_output
                                readOnly: true
                                wrapMode: TextEdit.Wrap
                                font.family: "Consolas, Monaco, monospace"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.text
                                selectByMouse: true

                                background: Rectangle {
                                    color: Theme.successBg
                                    border.color: Theme.success
                                    radius: Theme.inputRadius
                                }
                            }
                        }
                    }
                }

                // JSON Compare section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: compareColumn.implicitHeight + Theme.spacingLg * 2
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        id: compareColumn
                        anchors.fill: parent
                        anchors.margins: Theme.spacingLg
                        spacing: Theme.spacingMd

                        Label {
                            text: "Compare JSON"
                            font.bold: true
                            color: Theme.text
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingMd

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm

                                Label {
                                    text: "JSON A"
                                    color: Theme.textSecondary
                                }

                                ScrollView {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 100

                                    TextArea {
                                        text: jsonModel.diff_input_a
                                        placeholderText: "First JSON..."
                                        wrapMode: TextEdit.Wrap
                                        font.family: "Consolas, Monaco, monospace"
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.text
                                        placeholderTextColor: Theme.textMuted
                                        onTextChanged: jsonModel.diff_input_a = text

                                        background: Rectangle {
                                            color: Theme.inputBg
                                            radius: Theme.inputRadius
                                        }
                                    }
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm

                                Label {
                                    text: "JSON B"
                                    color: Theme.textSecondary
                                }

                                ScrollView {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 100

                                    TextArea {
                                        text: jsonModel.diff_input_b
                                        placeholderText: "Second JSON..."
                                        wrapMode: TextEdit.Wrap
                                        font.family: "Consolas, Monaco, monospace"
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.text
                                        placeholderTextColor: Theme.textMuted
                                        onTextChanged: jsonModel.diff_input_b = text

                                        background: Rectangle {
                                            color: Theme.inputBg
                                            radius: Theme.inputRadius
                                        }
                                    }
                                }
                            }
                        }

                        Button {
                            text: "Compare"
                            onClicked: jsonModel.compare_json()

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

                        Rectangle {
                            visible: jsonModel.diff_result.length > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: diffResultColumn.implicitHeight + Theme.spacingMd * 2
                            color: jsonModel.diff_result.startsWith("identical") ? Theme.successBg : Theme.warningBg
                            border.color: jsonModel.diff_result.startsWith("identical") ? Theme.success : Theme.warning
                            radius: Theme.cardRadius

                            ColumnLayout {
                                id: diffResultColumn
                                anchors.fill: parent
                                anchors.margins: Theme.spacingMd
                                spacing: Theme.spacingSm

                                RowLayout {
                                    spacing: Theme.spacingMd

                                    Label {
                                        text: jsonModel.diff_result.startsWith("identical") ? Icons.check : Icons.warning
                                        font.family: Icons.family
                                        font.pixelSize: 20
                                        color: jsonModel.diff_result.startsWith("identical") ? Theme.success : Theme.warning
                                    }

                                    Label {
                                        text: jsonModel.diff_result.startsWith("identical") ? "Documents are identical" : "Documents differ"
                                        font.bold: true
                                        color: jsonModel.diff_result.startsWith("identical") ? Theme.success : Theme.warning
                                    }
                                }

                                Label {
                                    visible: jsonModel.diff_result.startsWith("different:")
                                    text: jsonModel.diff_result.substring(10)
                                    wrapMode: Text.WordWrap
                                    Layout.fillWidth: true
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                }
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }
    }

    // Text Chunker Component
    Component {
        id: chunkerToolComponent

        Item {
            anchors.fill: parent

            // State
            property string inputText: ""
            property var chunks: []
            property int maxChunkSize: 10000
            property int copiedChunkIndex: -1
            property bool prefixEnabled: true
            property string prefixText: "help me with my new claude code brainstorming session. i prefer you give me replies to claude code i can copy and paste. ===== i'm open to suggestions and questions."

            // Smart chunking function - breaks at natural boundaries
            function chunkText(text, maxSize) {
                if (!text || text.length === 0 || maxSize <= 0) {
                    return [];
                }

                const result = [];
                let remaining = text;
                const minBreakPoint = maxSize * 0.5;

                while (remaining.length > 0) {
                    if (remaining.length <= maxSize) {
                        result.push(remaining);
                        break;
                    }

                    // Find the best break point within maxSize
                    let breakPoint = maxSize;
                    const searchArea = remaining.substring(0, maxSize);

                    // Try paragraph break first (\n\n)
                    const paragraphBreak = searchArea.lastIndexOf("\n\n");
                    if (paragraphBreak > minBreakPoint) {
                        breakPoint = paragraphBreak + 2;
                    } else {
                        // Try single newline
                        const lineBreak = searchArea.lastIndexOf("\n");
                        if (lineBreak > minBreakPoint) {
                            breakPoint = lineBreak + 1;
                        } else {
                            // Try sentence break (". " or ".\n")
                            const dotSpace = searchArea.lastIndexOf(". ");
                            const dotNewline = searchArea.lastIndexOf(".\n");
                            const sentenceBreak = Math.max(dotSpace, dotNewline);
                            if (sentenceBreak > minBreakPoint) {
                                breakPoint = sentenceBreak + 2;
                            } else {
                                // Try word break (space)
                                const wordBreak = searchArea.lastIndexOf(" ");
                                if (wordBreak > minBreakPoint) {
                                    breakPoint = wordBreak + 1;
                                }
                                // Otherwise, hard cut at maxSize
                            }
                        }
                    }

                    result.push(remaining.substring(0, breakPoint));
                    remaining = remaining.substring(breakPoint);
                }

                // Add instructions if multiple chunks
                if (result.length > 1) {
                    const total = result.length;
                    const remainingChunks = total - 1;

                    // First chunk - add "wait for more" header
                    result[0] = `[CHUNK 1/${total}] WAIT FOR ${remainingChunks} MORE PASTE${remainingChunks > 1 ? 'S' : ''} THEN REVIEW ALL PLEASE\n\n` + result[0];

                    // Middle chunks - add chunk number header
                    for (let i = 1; i < total - 1; i++) {
                        result[i] = `[CHUNK ${i + 1}/${total}]\n\n` + result[i];
                    }

                    // Last chunk - add header and "review all" footer
                    result[total - 1] = `[CHUNK ${total}/${total}]\n\n` + result[total - 1] + `\n\n--- THIS IS THE LAST CHUNK - PLEASE REVIEW ALL NOW ---`;
                }

                // Prepend prefix to first chunk if enabled
                if (prefixEnabled && prefixText.length > 0 && result.length > 0) {
                    result[0] = prefixText + "\n\n" + result[0];
                }

                return result;
            }

            // Debounce chunking for large text performance
            Timer {
                id: chunkingTimer
                interval: 150
                onTriggered: chunks = chunkText(inputText, maxChunkSize)
            }

            // Update chunks when input or prefix settings change (debounced)
            onInputTextChanged: chunkingTimer.restart()
            onPrefixEnabledChanged: chunkingTimer.restart()
            onPrefixTextChanged: chunkingTimer.restart()

            // Copy feedback timer
            Timer {
                id: copyFeedbackTimer
                interval: 2000
                onTriggered: copiedChunkIndex = -1
            }

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingLg
                spacing: Theme.spacingLg

                // Left column - Input area
                Rectangle {
                    Layout.fillHeight: true
                    Layout.preferredWidth: parent.width * 0.45
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        // Header
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingSm

                            Label {
                                text: "Input Text"
                                font.pixelSize: Theme.fontSizeMedium
                                font.bold: true
                                color: Theme.text
                                Layout.fillWidth: true
                            }

                            // Character count
                            Label {
                                text: inputText.length.toLocaleString() + " chars"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }

                            // Clear button
                            Rectangle {
                                visible: inputText.length > 0
                                width: 28
                                height: 28
                                radius: Theme.buttonRadius
                                color: clearMouseArea.containsMouse ? Theme.errorBg : "transparent"

                                Label {
                                    anchors.centerIn: parent
                                    text: Icons.trash
                                    font.family: Icons.family
                                    font.pixelSize: 16
                                    color: clearMouseArea.containsMouse ? Theme.error : Theme.textSecondary
                                }

                                MouseArea {
                                    id: clearMouseArea
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: inputText = ""
                                }

                                ToolTip.visible: clearMouseArea.containsMouse
                                ToolTip.text: "Clear input"
                                ToolTip.delay: 500
                            }
                        }

                        // Prefix section
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: prefixColumn.implicitHeight + Theme.spacingSm * 2
                            color: prefixEnabled ? Theme.primary + "10" : Theme.surfaceAlt
                            border.color: prefixEnabled ? Theme.primary + "40" : (Theme.isDark ? "#ffffff08" : "#00000008")
                            radius: Theme.cardRadius

                            Behavior on color { ColorAnimation { duration: 150 } }
                            Behavior on border.color { ColorAnimation { duration: 150 } }

                            ColumnLayout {
                                id: prefixColumn
                                anchors.fill: parent
                                anchors.margins: Theme.spacingSm
                                spacing: Theme.spacingSm

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: Theme.spacingSm

                                    Switch {
                                        id: prefixSwitch
                                        checked: prefixEnabled
                                        onCheckedChanged: prefixEnabled = checked
                                    }

                                    Label {
                                        text: "Prepend prefix to first chunk"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.bold: true
                                        color: prefixEnabled ? Theme.text : Theme.textSecondary
                                        Layout.fillWidth: true

                                        MouseArea {
                                            anchors.fill: parent
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: prefixSwitch.checked = !prefixSwitch.checked
                                        }
                                    }
                                }

                                Rectangle {
                                    visible: prefixEnabled
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 60
                                    color: Theme.inputBg
                                    border.color: prefixInput.activeFocus ? Theme.primary : Theme.inputBorder
                                    border.width: prefixInput.activeFocus ? 2 : 1
                                    radius: Theme.inputRadius

                                    Behavior on border.color { ColorAnimation { duration: 100 } }

                                    ScrollView {
                                        anchors.fill: parent
                                        anchors.margins: 2

                                        TextArea {
                                            id: prefixInput
                                            text: prefixText
                                            wrapMode: TextEdit.Wrap
                                            font.family: "Consolas, Monaco, monospace"
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.text
                                            placeholderText: "Text to prepend to the first chunk..."
                                            placeholderTextColor: Theme.textMuted
                                            onTextChanged: prefixText = text

                                            background: Rectangle { color: "transparent" }
                                        }
                                    }
                                }
                            }
                        }

                        // Input TextArea
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            color: Theme.inputBg
                            border.color: inputArea.activeFocus ? Theme.primary : Theme.inputBorder
                            border.width: inputArea.activeFocus ? 2 : 1
                            radius: Theme.inputRadius

                            Behavior on border.color {
                                ColorAnimation { duration: 100 }
                            }

                            ScrollView {
                                anchors.fill: parent
                                anchors.margins: 2

                                TextArea {
                                    id: inputArea
                                    text: inputText
                                    placeholderText: "Paste your large text here...\n\nThe text will be automatically split into chunks of up to " + maxChunkSize.toLocaleString() + " characters, breaking at natural boundaries (paragraphs, sentences, words)."
                                    wrapMode: TextEdit.Wrap
                                    font.family: "Consolas, Monaco, monospace"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.text
                                    placeholderTextColor: Theme.textMuted
                                    onTextChanged: inputText = text

                                    background: Rectangle {
                                        color: "transparent"
                                    }
                                }
                            }
                        }

                        // Info bar
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 36
                            color: Theme.infoBg
                            border.color: Theme.info
                            radius: Theme.buttonRadius
                            visible: inputText.length > 0

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingSm
                                spacing: Theme.spacingSm

                                Label {
                                    text: Icons.info
                                    font.family: Icons.family
                                    font.pixelSize: 16
                                    color: Theme.info
                                }

                                Label {
                                    text: chunks.length + " chunk" + (chunks.length !== 1 ? "s" : "") + " • Max " + maxChunkSize.toLocaleString() + " chars each"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.info
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }
                }

                // Right column - Chunks display
                Rectangle {
                    Layout.fillHeight: true
                    Layout.fillWidth: true
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    radius: Theme.cardRadius

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        // Header
                        Label {
                            text: "Chunks"
                            font.pixelSize: Theme.fontSizeMedium
                            font.bold: true
                            color: Theme.text
                        }

                        // Empty state
                        ColumnLayout {
                            visible: chunks.length === 0
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            spacing: Theme.spacingMd

                            Item { Layout.fillHeight: true }

                            Label {
                                text: Icons.scissors
                                font.family: Icons.family
                                font.pixelSize: 48
                                color: Theme.textMuted
                                Layout.alignment: Qt.AlignHCenter
                            }

                            Label {
                                text: "No text to chunk"
                                font.pixelSize: Theme.fontSizeLarge
                                font.bold: true
                                color: Theme.text
                                Layout.alignment: Qt.AlignHCenter
                            }

                            Label {
                                text: "Paste text in the left panel to split it into copyable chunks"
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textSecondary
                                Layout.alignment: Qt.AlignHCenter
                            }

                            Item { Layout.fillHeight: true }
                        }

                        // Chunks list
                        ScrollView {
                            visible: chunks.length > 0
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            clip: true

                            ColumnLayout {
                                width: parent.width
                                spacing: Theme.spacingMd

                                Repeater {
                                    model: chunks

                                    Rectangle {
                                        required property int index
                                        required property string modelData

                                        Layout.fillWidth: true
                                        Layout.preferredHeight: chunkContent.implicitHeight + Theme.spacingMd * 2
                                        color: copiedChunkIndex === index ? Theme.successBg : Theme.surfaceAlt
                                        border.color: copiedChunkIndex === index ? Theme.success : (Theme.isDark ? "#ffffff08" : "#00000008")
                                        border.width: copiedChunkIndex === index ? 2 : 1
                                        radius: Theme.cardRadius

                                        Behavior on color {
                                            ColorAnimation { duration: 150 }
                                        }

                                        Behavior on border.color {
                                            ColorAnimation { duration: 150 }
                                        }

                                        ColumnLayout {
                                            id: chunkContent
                                            anchors.fill: parent
                                            anchors.margins: Theme.spacingMd
                                            spacing: Theme.spacingSm

                                            // Chunk header
                                            RowLayout {
                                                Layout.fillWidth: true
                                                spacing: Theme.spacingSm

                                                // Chunk number badge
                                                Rectangle {
                                                    width: chunkLabel.implicitWidth + Theme.spacingSm * 2
                                                    height: chunkLabel.implicitHeight + Theme.spacingXs
                                                    radius: 4
                                                    color: Theme.primary + "20"

                                                    Label {
                                                        id: chunkLabel
                                                        anchors.centerIn: parent
                                                        text: "Chunk " + (index + 1) + "/" + chunks.length
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.bold: true
                                                        color: Theme.primary
                                                    }
                                                }

                                                // Character count
                                                Label {
                                                    text: modelData.length.toLocaleString() + " chars"
                                                    font.pixelSize: Theme.fontSizeSmall
                                                    color: Theme.textSecondary
                                                }

                                                Item { Layout.fillWidth: true }

                                                // Copy button
                                                Button {
                                                    id: copyBtn
                                                    text: copiedChunkIndex === index ? (Icons.check + " Copied!") : (Icons.copy + " Copy")
                                                    font.family: Icons.family

                                                    onClicked: {
                                                        // Use hidden TextArea for copying
                                                        hiddenCopyArea.text = modelData;
                                                        hiddenCopyArea.selectAll();
                                                        hiddenCopyArea.copy();
                                                        hiddenCopyArea.deselect();
                                                        copiedChunkIndex = index;
                                                        copyFeedbackTimer.restart();
                                                    }

                                                    background: Rectangle {
                                                        radius: Theme.buttonRadius
                                                        color: copiedChunkIndex === index ? Theme.success : (parent.hovered ? Theme.primaryHover : Theme.primary)
                                                    }

                                                    contentItem: Text {
                                                        text: parent.text
                                                        font.family: Icons.family
                                                        color: Theme.primaryText
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.bold: true
                                                        horizontalAlignment: Text.AlignHCenter
                                                        verticalAlignment: Text.AlignVCenter
                                                    }
                                                }
                                            }

                                            // Chunk preview
                                            Rectangle {
                                                Layout.fillWidth: true
                                                Layout.preferredHeight: 100
                                                color: Theme.inputBg
                                                border.color: Theme.inputBorder
                                                radius: Theme.inputRadius

                                                ScrollView {
                                                    anchors.fill: parent
                                                    anchors.margins: Theme.spacingSm

                                                    TextArea {
                                                        text: modelData
                                                        readOnly: true
                                                        wrapMode: TextEdit.Wrap
                                                        font.family: "Consolas, Monaco, monospace"
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        color: Theme.text
                                                        selectByMouse: true

                                                        background: Rectangle {
                                                            color: "transparent"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                Item { height: Theme.spacingMd }
                            }
                        }
                    }
                }
            }

            // Hidden TextArea for clipboard operations
            TextArea {
                id: hiddenCopyArea
                visible: false
            }
        }
    }
}
