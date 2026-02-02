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

    Timer {
        id: pollTimer
        interval: 100
        running: calendarModel.loading
        repeat: true
        onTriggered: calendarModel.poll_channel()
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
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
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
                    font.pixelSize: Theme.fontSizeSmall
                    font.bold: true
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
                font.pixelSize: Theme.fontSizeLarge
                color: Theme.text
                Layout.alignment: Qt.AlignHCenter
            }

            Label {
                text: "Sign in to view your calendar events"
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
                    console.log("Navigate to settings")
                }
            }
        }
    }

    // Event list
    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        visible: calendarModel.authenticated
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingMd

            // Today section header
            Label {
                text: "Upcoming Events"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }

            // Events
            Repeater {
                model: calendarModel.event_count

                Rectangle {
                    Layout.fillWidth: true
                    height: eventContent.implicitHeight + Theme.spacingMd * 2
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: Theme.border
                    border.width: 1

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
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: true
                                    color: Theme.primary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    visible: eventData.allDay
                                    text: "Day"
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
                                font.pixelSize: Theme.fontSizeNormal
                                font.bold: true
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
            Item {
                visible: calendarModel.event_count === 0 && !calendarModel.loading
                Layout.fillWidth: true
                Layout.preferredHeight: 200

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingMd

                    Text {
                        text: Icons.calendarCheck
                        font.family: Icons.family
                        font.pixelSize: 48
                        color: Theme.textMuted
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "No upcoming events"
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.textSecondary
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Your schedule is clear for the next 7 days"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textMuted
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: calendarModel.loading
        visible: calendarModel.loading
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
        visible: calendarModel.error_message !== ""

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
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.error
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }
    }
}
