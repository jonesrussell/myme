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
            id: "base64",
            name: "Base64 Encoder",
            description: "Encode and decode Base64 strings",
            icon: Icons.code,
            category: "Encoding",
            comingSoon: true
        },
        {
            id: "uuid",
            name: "UUID Generator",
            description: "Generate random UUIDs (v4) for testing",
            icon: Icons.squaresFour,
            category: "Generators",
            comingSoon: true
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
            description: "Generate MD5, SHA-1, SHA-256 hashes from text",
            icon: Icons.lock,
            category: "Security",
            comingSoon: true
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
        var query = searchQuery.toLowerCase();
        return tools.filter(function(tool) {
            return tool.name.toLowerCase().indexOf(query) !== -1 ||
                   tool.description.toLowerCase().indexOf(query) !== -1 ||
                   tool.category.toLowerCase().indexOf(query) !== -1;
        });
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
                    for (var i = 0; i < tools.length; i++) {
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
                                var alg = jwtModel.algorithm;
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
}
