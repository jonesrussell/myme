import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: devToolsPage
    title: "Developer Tools"

    background: Rectangle {
        color: Theme.background
    }

    // Instantiate the JwtModel from Rust
    JwtModel {
        id: jwtModel
        payload: '{\n  "sub": "user123",\n  "name": "John Doe",\n  "iat": 1234567890\n}'
        algorithm: "HS256"
    }

    header: ToolBar {
        background: Rectangle {
            color: Theme.surface
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Developer Tools"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }
        }
    }

    // Main content
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

                    // Section header
                    RowLayout {
                        spacing: Theme.spacingSm

                        Label {
                            text: "🔐"
                            font.pixelSize: 20
                        }

                        Label {
                            text: "JWT Token Generator"
                            font.pixelSize: Theme.fontSizeMedium
                            font.bold: true
                            color: Theme.text
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Theme.border
                    }

                    // Error message banner
                    Rectangle {
                        visible: jwtModel.error_message.length > 0
                        Layout.fillWidth: true
                        Layout.preferredHeight: 50
                        color: Theme.isDark ? "#4a1a1a" : "#FFE6E6"
                        border.color: Theme.error
                        border.width: 1
                        radius: Theme.cardRadius

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMd
                            spacing: Theme.spacingMd

                            Label {
                                text: "⚠"
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
                            ColorAnimation { duration: 100 }
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
                                ColorAnimation { duration: 100 }
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
                            var alg = jwtModel.algorithm
                            if (alg === "HS384") return 1
                            if (alg === "HS512") return 2
                            return 0
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
                        color: Theme.isDark ? "#1a3a1a" : "#E8F5E9"
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
                                tokenOutput.selectAll()
                                tokenOutput.copy()
                                tokenOutput.deselect()
                                copyFeedback.visible = true
                                copyFeedbackTimer.start()
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
                                text: "✓ Copied!"
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
                        color: Theme.isDark ? "#1a2a4a" : "#E3F2FD"
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
