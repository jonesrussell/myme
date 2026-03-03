import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: calendarPage
    title: "Calendar"

    CalendarModel {
        id: calendarModel
        Component.onCompleted: {
            calendarModel.check_auth()
            if (calendarModel.authenticated) {
                calendarModel.fetch_events()
            }
        }
    }

    // Poll timer: runs while loading or syncing
    Timer {
        id: pollTimer
        interval: 100
        running: calendarModel.loading || calendarModel.syncing
        repeat: true
        onTriggered: calendarModel.poll_channel()
    }

    // Background sync timer
    Timer {
        id: syncTimer
        interval: calendarModel.sync_interval * 1000
        running: calendarModel.authenticated && calendarModel.sync_interval > 0
        repeat: true
        onTriggered: calendarModel.background_sync()
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
                text: "Calendar"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Bold
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingLg
            }

            // Sync indicator (subtle pulsing dot during background sync)
            Rectangle {
                visible: calendarModel.syncing
                width: 8
                height: 8
                radius: 4
                color: Theme.primary
                opacity: syncPulse.running ? 1 : 0.4

                SequentialAnimation on opacity {
                    id: syncPulse
                    running: calendarModel.syncing
                    loops: Animation.Infinite
                    NumberAnimation { to: 0.3; duration: 800; easing.type: Easing.InOutSine }
                    NumberAnimation { to: 1.0; duration: 800; easing.type: Easing.InOutSine }
                }
            }

            // Last synced label
            Label {
                visible: calendarModel.last_synced !== ""
                text: "Synced " + calendarModel.last_synced
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
            }

            // Today's event count badge
            Rectangle {
                visible: calendarModel.today_event_count > 0
                Layout.preferredWidth: 32
                Layout.preferredHeight: 24
                radius: 12
                color: Theme.primary

                Label {
                    anchors.centerIn: parent
                    text: calendarModel.today_event_count
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.Bold
                    color: Theme.primaryText
                }
            }

            Button {
                text: calendarModel.loading ? "Refreshing..." : "Refresh"
                enabled: !calendarModel.loading && calendarModel.authenticated
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

                onClicked: calendarModel.fetch_events()
            }
        }
    }

    // Not authenticated state
    Item {
        anchors.fill: parent
        visible: !calendarModel.authenticated

        ColumnLayout {
            anchors.centerIn: parent
            spacing: Theme.spacingLg

            Text {
                text: Icons.calendarBlank
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
                text: "Sign in to view your calendar events"
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

    // Event list
    ScrollView {
        id: calendarScroll
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        visible: calendarModel.authenticated
        clip: true
        contentWidth: calendarScroll.viewport.width

        ColumnLayout {
            width: calendarScroll.viewport.width
            spacing: Theme.spacingMd

            // Today section header
            Label {
                text: "Upcoming Events"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeMedium
                font.weight: Font.Bold
                color: Theme.text
            }

            // Events
            Repeater {
                model: calendarModel.event_count

                Rectangle {
                    id: eventDelegate
                    required property int index
                    Layout.fillWidth: true
                    height: eventContent.implicitHeight + Theme.spacingMd * 2
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: Theme.cardBorderSubtle
                    border.width: 1

                    opacity: 0
                    Component.onCompleted: eventEntryAnim.start()
                    SequentialAnimation {
                        id: eventEntryAnim
                        PauseAnimation { duration: eventDelegate.index * 30 }
                        ParallelAnimation {
                            NumberAnimation { target: eventDelegate; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                            NumberAnimation { target: eventDelegate; property: "y"; from: eventDelegate.y + 8; to: eventDelegate.y; duration: 200; easing.type: Easing.OutCubic }
                        }
                    }

                    property var eventData: {
                        try {
                            return JSON.parse(calendarModel.get_event(index))
                        } catch (e) {
                            return {}
                        }
                    }

                    RowLayout {
                        id: eventContent
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingMd

                        // Time indicator
                        Rectangle {
                            Layout.preferredWidth: 60
                            Layout.preferredHeight: 50
                            radius: Theme.cardRadius
                            color: Theme.primary + "20"

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 0

                                Label {
                                    text: {
                                        if (!eventData.start) return "--:--"
                                        if (eventData.allDay) return "All"
                                        const d = new Date(eventData.start)
                                        return d.toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'})
                                    }
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.weight: Font.Bold
                                    color: Theme.primary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    visible: eventData.allDay
                                    text: "Day"
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.primary
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Label {
                                text: eventData.summary || "(No title)"
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSizeNormal
                                font.weight: Font.Bold
                                color: Theme.text
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            RowLayout {
                                visible: eventData.start
                                spacing: Theme.spacingSm

                                Text {
                                    text: Icons.clock
                                    font.family: Icons.family
                                    font.pixelSize: 14
                                    color: Theme.textSecondary
                                }

                                Label {
                                    text: {
                                        if (!eventData.start) return ""
                                        const start = new Date(eventData.start)
                                        const end = new Date(eventData.end)
                                        if (eventData.allDay) {
                                            return start.toLocaleDateString()
                                        }
                                        return start.toLocaleDateString() + " • " +
                                               start.toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'}) +
                                               " - " +
                                               end.toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'})
                                    }
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                            }

                            RowLayout {
                                visible: eventData.location
                                spacing: Theme.spacingSm

                                Text {
                                    text: Icons.mapPin
                                    font.family: Icons.family
                                    font.pixelSize: 14
                                    color: Theme.textSecondary
                                }

                                Label {
                                    text: eventData.location || ""
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                            }
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            // Could open event details
                            console.log("Event clicked:", eventData.id)
                        }
                    }
                }
            }

            // Empty state
            ColumnLayout {
                visible: calendarModel.event_count === 0 && !calendarModel.loading
                Layout.fillWidth: true
                Layout.preferredHeight: 200
                Layout.alignment: Qt.AlignHCenter

                Item { Layout.fillHeight: true }

                Text {
                    text: Icons.calendarCheck
                    font.family: Icons.family
                    font.pixelSize: 48
                    color: Theme.textMuted
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "No upcoming events"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeMedium
                    font.weight: Font.Bold
                    color: Theme.text
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Your schedule is clear for the next 7 days"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textMuted
                    Layout.alignment: Qt.AlignHCenter
                }

                Item { Layout.fillHeight: true }
            }
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: calendarModel.loading
        visible: calendarModel.loading
    }

    // Error message (inline, at top)
    Rectangle {
        visible: calendarModel.error_message !== ""
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
                text: calendarModel.error_message
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.error
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }
    }
}
