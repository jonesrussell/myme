import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui

Page {
    id: devToolsPage
    title: "Developer Tools"

    // Instantiate the JwtModel from Rust
    JwtModel {
        id: jwtModel
        payload: '{\n  "sub": "user123",\n  "name": "John Doe",\n  "iat": 1234567890\n}'
        algorithm: "HS256"
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            spacing: 10

            Label {
                text: "Developer Tools"
                font.pixelSize: 18
                font.bold: true
                Layout.fillWidth: true
                leftPadding: 10
            }
        }
    }

    // Main content
    ScrollView {
        anchors.fill: parent
        anchors.margins: 10
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: 20

            // JWT Generator Section
            GroupBox {
                title: "JWT Token Generator"
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 15

                    // Error message banner
                    Rectangle {
                        visible: jwtModel.error_message.length > 0
                        Layout.fillWidth: true
                        Layout.preferredHeight: 50
                        color: "#FFE6E6"
                        border.color: "#FF4444"
                        border.width: 1
                        radius: 4

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 10
                            spacing: 10

                            Label {
                                text: "\u26A0"
                                font.pixelSize: 20
                                color: "#FF4444"
                            }

                            Label {
                                text: jwtModel.error_message
                                color: "#CC0000"
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                            }
                        }
                    }

                    // Payload input
                    Label {
                        text: "Payload (JSON):"
                        font.bold: true
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 150
                        border.color: "#E0E0E0"
                        border.width: 1
                        radius: 4

                        ScrollView {
                            anchors.fill: parent
                            anchors.margins: 2

                            TextArea {
                                id: payloadField
                                text: jwtModel.payload
                                placeholderText: '{\n  "sub": "user123",\n  "name": "John Doe",\n  "iat": 1234567890\n}'
                                wrapMode: TextEdit.Wrap
                                font.family: "monospace"
                                onTextChanged: jwtModel.payload = text
                            }
                        }
                    }

                    // Secret key input
                    Label {
                        text: "Secret Key:"
                        font.bold: true
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        TextField {
                            id: secretField
                            Layout.fillWidth: true
                            placeholderText: "Enter your secret key..."
                            echoMode: showSecretCheckbox.checked ? TextInput.Normal : TextInput.Password
                            text: jwtModel.secret
                            onTextChanged: jwtModel.secret = text
                        }

                        CheckBox {
                            id: showSecretCheckbox
                            text: "Show"
                        }
                    }

                    // Algorithm selector
                    Label {
                        text: "Algorithm:"
                        font.bold: true
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
                        Layout.preferredWidth: 200
                        highlighted: true
                        onClicked: jwtModel.generate_token()
                    }

                    // Generated token output
                    Label {
                        text: "Generated Token:"
                        font.bold: true
                        visible: jwtModel.generated_token.length > 0
                    }

                    Rectangle {
                        visible: jwtModel.generated_token.length > 0
                        Layout.fillWidth: true
                        Layout.preferredHeight: 120
                        color: "#F5F5F5"
                        border.color: "#4CAF50"
                        border.width: 2
                        radius: 4

                        ScrollView {
                            anchors.fill: parent
                            anchors.margins: 10

                            TextArea {
                                id: tokenOutput
                                text: jwtModel.generated_token
                                readOnly: true
                                wrapMode: TextEdit.WrapAnywhere
                                font.family: "monospace"
                                font.pixelSize: 12
                                selectByMouse: true
                            }
                        }
                    }

                    // Copy button
                    RowLayout {
                        visible: jwtModel.generated_token.length > 0
                        spacing: 10

                        Button {
                            text: "Copy to Clipboard"
                            onClicked: {
                                tokenOutput.selectAll()
                                tokenOutput.copy()
                                tokenOutput.deselect()
                                copyFeedback.visible = true
                                copyFeedbackTimer.start()
                            }
                        }

                        Label {
                            id: copyFeedback
                            text: "Copied!"
                            color: "#4CAF50"
                            font.bold: true
                            visible: false

                            Timer {
                                id: copyFeedbackTimer
                                interval: 2000
                                onTriggered: copyFeedback.visible = false
                            }
                        }
                    }

                    // Token info
                    Rectangle {
                        visible: jwtModel.generated_token.length > 0
                        Layout.fillWidth: true
                        Layout.preferredHeight: infoColumn.height + 20
                        color: "#E3F2FD"
                        border.color: "#2196F3"
                        border.width: 1
                        radius: 4

                        ColumnLayout {
                            id: infoColumn
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: 10
                            spacing: 5

                            Label {
                                text: "Token Information"
                                font.bold: true
                                color: "#1565C0"
                            }

                            Label {
                                text: "Algorithm: " + jwtModel.algorithm
                                font.pixelSize: 12
                            }

                            Label {
                                text: "Token Length: " + jwtModel.generated_token.length + " characters"
                                font.pixelSize: 12
                            }

                            Label {
                                text: "Parts: " + jwtModel.generated_token.split(".").length + " (Header.Payload.Signature)"
                                font.pixelSize: 12
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
