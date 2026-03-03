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

    // Poll timer: runs while loading or syncing
    Timer {
        id: pollTimer
        interval: 100
        running: gmailModel.loading || gmailModel.syncing
        repeat: true
        onTriggered: gmailModel.poll_channel()
    }

    // Background sync timer
    Timer {
        id: syncTimer
        interval: gmailModel.sync_interval * 1000
        running: gmailModel.authenticated && gmailModel.sync_interval > 0
        repeat: true
        onTriggered: gmailModel.background_sync()
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
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Bold
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingLg
            }

            // Sync indicator (subtle pulsing dot during background sync)
            Rectangle {
                visible: gmailModel.syncing
                width: 8
                height: 8
                radius: 4
                color: Theme.primary
                opacity: syncPulse.running ? 1 : 0.4

                SequentialAnimation on opacity {
                    id: syncPulse
                    running: gmailModel.syncing
                    loops: Animation.Infinite
                    NumberAnimation { to: 0.3; duration: 800; easing.type: Easing.InOutSine }
                    NumberAnimation { to: 1.0; duration: 800; easing.type: Easing.InOutSine }
                }
            }

            // Last synced label
            Label {
                visible: gmailModel.last_synced !== ""
                text: "Synced " + gmailModel.last_synced
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
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
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.Bold
                    color: Theme.primaryText
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
                    font.family: Theme.fontFamily
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
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.text
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Sign in to access your Gmail inbox"
                font.family: Theme.fontFamily
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
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.primaryText
                    horizontalAlignment: Text.AlignHCenter
                }

                onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("SettingsPage"))
            }
        }
    }

    // Message list
    ScrollView {
        id: gmailScroll
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        visible: gmailModel.authenticated
        clip: true
        contentWidth: gmailScroll.viewport.width

        ListView {
            id: messageList
            width: gmailScroll.viewport.width
            model: gmailModel.message_count
            spacing: Theme.spacingSm

            delegate: Rectangle {
                id: messageDelegate
                required property int index
                width: messageList.width
                height: 80
                radius: Theme.cardRadius
                color: Theme.surface
                border.color: Theme.cardBorderSubtle
                border.width: 1

                opacity: 0
                Component.onCompleted: msgEntryAnim.start()
                SequentialAnimation {
                    id: msgEntryAnim
                    PauseAnimation { duration: messageDelegate.index * 30 }
                    ParallelAnimation {
                        NumberAnimation { target: messageDelegate; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                        NumberAnimation { target: messageDelegate; property: "y"; from: messageDelegate.y + 8; to: messageDelegate.y; duration: 200; easing.type: Easing.OutCubic }
                    }
                }

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
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSizeNormal
                                font.weight: messageData.isUnread ? Font.Bold : Font.Normal
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
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }

                        Label {
                            text: messageData.subject || "(No subject)"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            font.weight: messageData.isUnread ? Font.Bold : Font.Normal
                            color: Theme.text
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        Label {
                            text: messageData.snippet || ""
                            font.family: Theme.fontFamily
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
                        color: messageData.isStarred ? Theme.warning : Theme.textMuted
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
            ColumnLayout {
                visible: messageList.count === 0 && !gmailModel.loading
                anchors.centerIn: parent
                spacing: Theme.spacingMd

                Text {
                    text: Icons.envelopeSimple
                    font.family: Icons.family
                    font.pixelSize: 48
                    color: Theme.textMuted
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "No messages"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeLarge
                    font.weight: Font.Bold
                    color: Theme.text
                    Layout.alignment: Qt.AlignHCenter
                }
            }
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: gmailModel.loading
        visible: gmailModel.loading
    }

    // Error message (inline, above message list)
    Rectangle {
        visible: gmailModel.error_message !== ""
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMd
        height: 50
        radius: Theme.cardRadius
        color: Theme.errorBg
        z: 10

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
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.error
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }
    }
}
