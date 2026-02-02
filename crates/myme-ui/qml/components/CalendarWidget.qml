import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import ".."

// Calendar widget for dashboard
// Shows today's event count and next event
Rectangle {
    id: root

    property bool loading: false
    property bool authenticated: false
    property int todayEventCount: 0
    property string nextEventSummary: ""
    property string nextEventTime: ""
    property string errorMessage: ""

    signal clicked()
    signal refreshRequested()

    color: Theme.cardBg
    border.color: Theme.cardBorder
    border.width: 1
    radius: Theme.cardRadius

    implicitWidth: 200
    implicitHeight: contentColumn.implicitHeight + Theme.cardPadding * 2

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        hoverEnabled: true
        onClicked: root.clicked()

        Rectangle {
            anchors.fill: parent
            color: parent.containsMouse ? Theme.surfaceHover : "transparent"
            radius: root.radius
            opacity: 0.5

            Behavior on color {
                ColorAnimation { duration: 150 }
            }
        }
    }

    ColumnLayout {
        id: contentColumn
        anchors.fill: parent
        anchors.margins: Theme.cardPadding
        spacing: Theme.spacingMd

        // Header
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            Text {
                font.pixelSize: Theme.fontSizeSmall
                font.weight: Font.Medium
                font.letterSpacing: 1
                text: "CALENDAR"
                color: Theme.textMuted
            }

            Item { Layout.fillWidth: true }

            // Refresh button
            Rectangle {
                width: 24
                height: 24
                radius: Theme.buttonRadius
                color: refreshArea.containsMouse ? Theme.surfaceHover : "transparent"
                visible: root.authenticated

                Text {
                    anchors.centerIn: parent
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeSmall
                    text: Icons.arrowsClockwise
                    color: Theme.textSecondary

                    RotationAnimation on rotation {
                        running: root.loading
                        from: 0
                        to: 360
                        duration: 1000
                        loops: Animation.Infinite
                    }
                }

                MouseArea {
                    id: refreshArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: mouse => {
                        mouse.accepted = true;
                        root.refreshRequested();
                    }
                }
            }
        }

        // Content based on auth state
        Loader {
            Layout.fillWidth: true
            sourceComponent: root.authenticated ? authenticatedContent : unauthenticatedContent
        }
    }

    // Authenticated content
    Component {
        id: authenticatedContent

        ColumnLayout {
            spacing: Theme.spacingMd

            RowLayout {
                spacing: Theme.spacingMd

                // Calendar icon with date
                Rectangle {
                    width: 44
                    height: 44
                    radius: 8
                    color: Theme.primary + "20"

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 0

                        Text {
                            font.pixelSize: Theme.fontSizeSmall
                            font.weight: Font.Bold
                            text: Qt.formatDate(new Date(), "ddd").toUpperCase()
                            color: Theme.primary
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Text {
                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Bold
                            text: Qt.formatDate(new Date(), "d")
                            color: Theme.primary
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    // Event count
                    Text {
                        font.pixelSize: Theme.fontSizeTitle
                        font.weight: Font.Bold
                        text: root.loading && root.todayEventCount === 0 ? "--" : root.todayEventCount.toString()
                        color: Theme.text
                    }

                    // Label
                    Text {
                        font.pixelSize: Theme.fontSizeSmall
                        text: root.todayEventCount === 1 ? "event today" : "events today"
                        color: Theme.textSecondary
                    }
                }
            }

            // Next event preview (if authenticated and has events)
            Rectangle {
                visible: root.nextEventSummary !== ""
                Layout.fillWidth: true
                height: nextEventContent.implicitHeight + Theme.spacingSm * 2
                radius: Theme.buttonRadius
                color: Theme.surfaceAlt

                RowLayout {
                    id: nextEventContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingSm
                    spacing: Theme.spacingSm

                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeSmall
                        text: Icons.clock
                        color: Theme.textSecondary
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        Text {
                            font.pixelSize: Theme.fontSizeSmall
                            font.weight: Font.Medium
                            text: root.nextEventSummary
                            color: Theme.text
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        Text {
                            font.pixelSize: Theme.fontSizeSmall
                            text: root.nextEventTime
                            color: Theme.textMuted
                        }
                    }
                }
            }
        }
    }

    // Unauthenticated content
    Component {
        id: unauthenticatedContent

        ColumnLayout {
            spacing: Theme.spacingSm

            Text {
                font.family: Icons.family
                font.pixelSize: 32
                text: Icons.calendarBlank
                color: Theme.textMuted
                Layout.alignment: Qt.AlignHCenter
            }

            Text {
                font.pixelSize: Theme.fontSizeSmall
                text: "Sign in to view events"
                color: Theme.textSecondary
                Layout.alignment: Qt.AlignHCenter
            }

            Text {
                font.pixelSize: Theme.fontSizeSmall
                text: "Connect account →"
                color: Theme.primary
                Layout.alignment: Qt.AlignHCenter
            }
        }
    }

    // Loading overlay
    Rectangle {
        anchors.fill: parent
        color: Theme.surface
        opacity: root.loading && !root.authenticated ? 0.8 : 0
        radius: root.radius
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation { duration: 200 }
        }

        ColumnLayout {
            anchors.centerIn: parent
            spacing: Theme.spacingSm

            Text {
                Layout.alignment: Qt.AlignHCenter
                font.family: Icons.family
                font.pixelSize: 24
                text: Icons.spinner
                color: Theme.textSecondary

                RotationAnimation on rotation {
                    running: root.loading
                    from: 0
                    to: 360
                    duration: 1000
                    loops: Animation.Infinite
                }
            }
        }
    }
}
