import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: gmailPage
    title: "Gmail"

    GmailModel {
        id: gmailModel
        Component.onCompleted: {
            gmailModel.check_auth()
            if (gmailModel.authenticated) {
                gmailModel.fetch_messages()
            }
        }
    }

    Timer {
        id: pollTimer
        interval: 100
        running: gmailModel.loading
        repeat: true
        onTriggered: gmailModel.poll_channel()
    }

    background: Rectangle {
        color: Theme.background
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Gmail"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }

            // Unread count badge
            Rectangle {
                visible: gmailModel.unread_count > 0
                Layout.preferredWidth: 32
                Layout.preferredHeight: 24
                radius: 12
                color: Theme.error

                Label {
                    anchors.centerIn: parent
                    text: gmailModel.unread_count
                    font.pixelSize: Theme.fontSizeSmall
                    font.bold: true
                    color: "#ffffff"
                }
            }

            Button {
                text: gmailModel.loading ? "Refreshing..." : "Refresh"
                enabled: !gmailModel.loading && gmailModel.authenticated
                Layout.rightMargin: Theme.spacingMd

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.enabled ? (parent.hovered ? Theme.primaryHover : Theme.primary) : Theme.surfaceAlt
                }

                contentItem: Label {
                    text: parent.text
                    font.pixelSize: Theme.fontSizeSmall
                    color: parent.enabled ? Theme.primaryText : Theme.textMuted
                    horizontalAlignment: Text.AlignHCenter
                }

                onClicked: gmailModel.fetch_messages()
            }
        }
    }

    // Not authenticated state
    Item {
        anchors.fill: parent
        visible: !gmailModel.authenticated

        ColumnLayout {
            anchors.centerIn: parent
            spacing: Theme.spacingLg

            Text {
                text: Icons.envelopeSimple
                font.family: Icons.family
                font.pixelSize: 64
                color: Theme.textMuted
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Connect your Google account"
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.text
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Sign in to access your Gmail inbox"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.textSecondary
                Layout.alignment: Qt.AlignHCenter
            }

            Button {
                text: "Go to Settings"
                Layout.alignment: Qt.AlignHCenter
                Layout.preferredWidth: 140

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                }

                contentItem: Label {
                    text: parent.text
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.primaryText
                    horizontalAlignment: Text.AlignHCenter
                }

                onClicked: {
                    // Navigate to settings - this would be handled by Main.qml
                    console.log("Navigate to settings")
                }
            }
        }
    }

    // Message list
    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        visible: gmailModel.authenticated
        clip: true

        ListView {
            id: messageList
            model: gmailModel.message_count
            spacing: Theme.spacingSm

            delegate: Rectangle {
                width: messageList.width
                height: 80
                radius: Theme.cardRadius
                color: Theme.surface
                border.color: Theme.border
                border.width: 1

                property var messageData: {
                    try {
                        return JSON.parse(gmailModel.get_message(index))
                    } catch (e) {
                        return {}
                    }
                }

                RowLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    // Unread indicator
                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        color: messageData.isUnread ? Theme.primary : "transparent"
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4

                        RowLayout {
                            Layout.fillWidth: true

                            Label {
                                text: messageData.from || "Unknown"
                                font.pixelSize: Theme.fontSizeNormal
                                font.bold: messageData.isUnread
                                color: Theme.text
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            Label {
                                text: {
                                    if (!messageData.date) return ""
                                    const d = new Date(messageData.date)
                                    return d.toLocaleDateString()
                                }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }

                        Label {
                            text: messageData.subject || "(No subject)"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: messageData.isUnread
                            color: Theme.text
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        Label {
                            text: messageData.snippet || ""
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                    }

                    // Star indicator
                    Text {
                        text: messageData.isStarred ? Icons.starFill : Icons.star
                        font.family: Icons.family
                        font.pixelSize: 20
                        color: messageData.isStarred ? "#f59e0b" : Theme.textMuted
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        // Mark as read when clicked
                        if (messageData.isUnread && messageData.id) {
                            gmailModel.mark_as_read(messageData.id)
                        }
                    }
                }
            }

            // Empty state
            Label {
                visible: messageList.count === 0 && !gmailModel.loading
                anchors.centerIn: parent
                text: "No messages"
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.textSecondary
            }
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: gmailModel.loading
        visible: gmailModel.loading
    }

    // Error message
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMd
        height: 50
        radius: Theme.cardRadius
        color: Theme.error + "20"
        border.color: Theme.error
        visible: gmailModel.error_message !== ""

        RowLayout {
            anchors.fill: parent
            anchors.margins: Theme.spacingSm

            Text {
                text: Icons.warning
                font.family: Icons.family
                font.pixelSize: 20
                color: Theme.error
            }

            Label {
                text: gmailModel.error_message
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.error
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }
    }
}
