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
            name: "JSON Formatter",
            description: "Format, validate, and minify JSON data",
            icon: Icons.terminalWindow,
            category: "Formatting",
            comingSoon: true
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
            name: "Timestamp Converter",
            description: "Convert between Unix timestamps and human-readable dates",
            icon: Icons.arrowsClockwise,
            category: "Conversion",
            comingSoon: true
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
            clip: true

            ColumnLayout {
                width: parent.width
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
                    columns: Math.max(1, Math.floor((devToolsPage.width - Theme.spacingLg * 2) / 280))
                    columnSpacing: Theme.spacingMd
                    rowSpacing: Theme.spacingMd

                    Repeater {
                        model: filteredTools()

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredWidth: 260
                            Layout.preferredHeight: 140
                            color: modelData.comingSoon ? Theme.surfaceAlt : (toolCardMouse.containsMouse ? Theme.surfaceHover : Theme.surface)
                            border.color: toolCardMouse.containsMouse && !modelData.comingSoon ? Theme.primary : Theme.border
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
            active: currentTool !== "index"
            sourceComponent: {
                if (currentTool === "jwt") return jwtToolComponent;
                if (currentTool === "encoding") return encodingToolComponent;
                if (currentTool === "uuid") return uuidToolComponent;
                if (currentTool === "hash") return hashToolComponent;
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
                    border.color: Theme.border
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
                            border.color: Theme.error
                            border.width: 1
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
                    border.color: Theme.border
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
                    border.color: Theme.border
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
                    border.color: Theme.border
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
                    border.color: Theme.border
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
}
