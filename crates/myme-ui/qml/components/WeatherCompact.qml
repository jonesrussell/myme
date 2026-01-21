import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import ".."

// Compact weather widget for sidebar footer
// Shows temperature + icon, with optional expanded state showing condition text
Item {
    id: root

    property bool expanded: true
    property bool loading: false
    property bool hasData: false
    property bool isStale: false

    property real temperature: 0
    property string condition: ""
    property string conditionIcon: "sun"

    signal clicked()

    implicitWidth: expanded ? expandedLayout.implicitWidth : compactLayout.implicitWidth
    implicitHeight: Math.max(32, expanded ? expandedLayout.implicitHeight : compactLayout.implicitHeight)

    // Helper function to get icon character from icon name
    function getWeatherIcon(iconName) {
        const iconMap = {
            "sun": Icons.sun,
            "cloud_sun": Icons.cloud_sun,
            "cloud": Icons.cloud,
            "cloud_fog": Icons.cloud_fog,
            "cloud_rain": Icons.cloud_rain,
            "cloud_snow": Icons.cloud_snow,
            "cloud_lightning": Icons.cloud_lightning
        };
        return iconMap[iconName] || Icons.sun;
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        hoverEnabled: true

        onClicked: root.clicked()

        Rectangle {
            anchors.fill: parent
            color: parent.containsMouse ? Theme.sidebarHover : "transparent"
            radius: Theme.buttonRadius

            Behavior on color {
                ColorAnimation { duration: 150 }
            }
        }
    }

    // Compact layout: just icon + temp
    RowLayout {
        id: compactLayout
        anchors.centerIn: parent
        spacing: Theme.spacingXs
        visible: !root.expanded

        Text {
            font.family: Icons.family
            font.pixelSize: Theme.fontSizeMedium
            text: getWeatherIcon(root.conditionIcon)
            color: Theme.textSecondary
            opacity: root.isStale ? 0.6 : 1.0
        }

        Text {
            font.pixelSize: Theme.fontSizeNormal
            font.weight: Font.Medium
            text: root.hasData ? `${Math.round(root.temperature)}°` : "--"
            color: Theme.text
            opacity: root.isStale ? 0.6 : 1.0
        }
    }

    // Expanded layout: icon + temp + condition text
    RowLayout {
        id: expandedLayout
        anchors.centerIn: parent
        spacing: Theme.spacingSm
        visible: root.expanded

        Text {
            font.family: Icons.family
            font.pixelSize: Theme.fontSizeLarge
            text: getWeatherIcon(root.conditionIcon)
            color: Theme.textSecondary
            opacity: root.isStale ? 0.6 : 1.0
        }

        ColumnLayout {
            spacing: 0

            Text {
                font.pixelSize: Theme.fontSizeMedium
                font.weight: Font.Medium
                text: root.hasData ? `${Math.round(root.temperature)}°` : "--"
                color: Theme.text
                opacity: root.isStale ? 0.6 : 1.0
            }

            Text {
                font.pixelSize: Theme.fontSizeSmall
                text: root.hasData ? root.condition : "Loading..."
                color: Theme.textSecondary
                opacity: root.isStale ? 0.6 : 1.0
                visible: root.expanded
            }
        }

        // Stale indicator
        Text {
            visible: root.isStale
            font.family: Icons.family
            font.pixelSize: Theme.fontSizeSmall
            text: Icons.warning
            color: Theme.warning

            ToolTip {
                id: staleTooltip
                visible: staleMouseArea.containsMouse
                text: "Weather data may be outdated"
                delay: 500
            }

            MouseArea {
                id: staleMouseArea
                anchors.fill: parent
                hoverEnabled: true
            }
        }
    }

    // Loading spinner overlay
    Text {
        anchors.centerIn: parent
        visible: root.loading
        font.family: Icons.family
        font.pixelSize: Theme.fontSizeMedium
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
