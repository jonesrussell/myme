import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import ".."

// Full weather widget for dashboard
// Shows complete current day information
Rectangle {
    id: root

    property bool loading: false
    property bool hasData: false
    property bool isStale: false

    // Current conditions
    property real temperature: 0
    property real feelsLike: 0
    property int humidity: 0
    property real windSpeed: 0
    property string condition: ""
    property string conditionIcon: "sun"
    property string locationName: ""

    // Today's forecast
    property real todayHigh: 0
    property real todayLow: 0
    property int precipChance: 0
    property string sunrise: ""
    property string sunset: ""

    signal clicked()
    signal refreshRequested()

    color: Theme.cardBg
    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
    border.width: 1
    radius: Theme.cardRadius

    implicitWidth: 320
    implicitHeight: contentColumn.implicitHeight + Theme.cardPadding * 2

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

        // Header with location and refresh
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.Medium
                    font.letterSpacing: 1
                    text: "WEATHER"
                    color: Theme.textMuted
                }

                Text {
                    font.pixelSize: Theme.fontSizeNormal
                    text: root.hasData ? root.locationName : "Unknown Location"
                    color: Theme.textSecondary
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                }
            }

            // Stale indicator
            Text {
                visible: root.isStale
                font.family: Icons.family
                font.pixelSize: Theme.fontSizeNormal
                text: Icons.warning
                color: Theme.warning

                ToolTip.visible: staleArea.containsMouse
                ToolTip.text: "Weather data may be outdated"
                ToolTip.delay: 500

                MouseArea {
                    id: staleArea
                    anchors.fill: parent
                    hoverEnabled: true
                }
            }

            // Refresh button
            Rectangle {
                width: 28
                height: 28
                radius: Theme.buttonRadius
                color: refreshArea.containsMouse ? Theme.surfaceHover : "transparent"

                Text {
                    anchors.centerIn: parent
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
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

        // Main temperature display
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingMd

            // Large weather icon
            Text {
                font.family: Icons.family
                font.pixelSize: 48
                text: getWeatherIcon(root.conditionIcon)
                color: Theme.primary
                opacity: root.isStale ? 0.6 : 1.0
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                // Temperature
                Text {
                    font.pixelSize: Theme.fontSizeTitle
                    font.weight: Font.Bold
                    text: root.hasData ? `${Math.round(root.temperature)}°` : "--"
                    color: Theme.text
                    opacity: root.isStale ? 0.6 : 1.0
                }

                // Condition
                Text {
                    font.pixelSize: Theme.fontSizeMedium
                    text: root.hasData ? root.condition : "Loading..."
                    color: Theme.textSecondary
                    opacity: root.isStale ? 0.6 : 1.0
                }

                // Feels like
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? `Feels like ${Math.round(root.feelsLike)}°` : ""
                    color: Theme.textMuted
                    opacity: root.isStale ? 0.6 : 1.0
                }
            }

            // High/Low
            ColumnLayout {
                spacing: 4

                RowLayout {
                    spacing: 4
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeSmall
                        text: Icons.caretUp
                        color: Theme.error
                    }
                    Text {
                        font.pixelSize: Theme.fontSizeNormal
                        text: root.hasData ? `${Math.round(root.todayHigh)}°` : "--"
                        color: Theme.text
                        opacity: root.isStale ? 0.6 : 1.0
                    }
                }

                RowLayout {
                    spacing: 4
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeSmall
                        text: Icons.caretDown
                        color: Theme.info
                    }
                    Text {
                        font.pixelSize: Theme.fontSizeNormal
                        text: root.hasData ? `${Math.round(root.todayLow)}°` : "--"
                        color: Theme.text
                        opacity: root.isStale ? 0.6 : 1.0
                    }
                }
            }
        }

        // Divider
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.borderLight
        }

        // Details row
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingMd

            // Humidity
            RowLayout {
                spacing: Theme.spacingXs
                Text {
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
                    text: Icons.drop
                    color: Theme.info
                }
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? `${root.humidity}%` : "--"
                    color: Theme.textSecondary
                }
            }

            // Wind
            RowLayout {
                spacing: Theme.spacingXs
                Text {
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
                    text: Icons.wind
                    color: Theme.textMuted
                }
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? `${Math.round(root.windSpeed)} mph` : "--"
                    color: Theme.textSecondary
                }
            }

            // Precipitation
            RowLayout {
                spacing: Theme.spacingXs
                Text {
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
                    text: Icons.cloud_rain
                    color: Theme.textMuted
                }
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? `${root.precipChance}%` : "--"
                    color: Theme.textSecondary
                }
            }
        }

        // Sunrise/Sunset row
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingLg

            // Sunrise
            RowLayout {
                spacing: Theme.spacingXs
                Text {
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
                    text: Icons.sun
                    color: Theme.warning
                }
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? root.sunrise : "--:--"
                    color: Theme.textSecondary
                }
            }

            // Sunset
            RowLayout {
                spacing: Theme.spacingXs
                Text {
                    font.family: Icons.family
                    font.pixelSize: Theme.fontSizeNormal
                    text: Icons.moon
                    color: Theme.textMuted
                }
                Text {
                    font.pixelSize: Theme.fontSizeSmall
                    text: root.hasData ? root.sunset : "--:--"
                    color: Theme.textSecondary
                }
            }

            Item { Layout.fillWidth: true }

            // "View details" hint
            Text {
                font.pixelSize: Theme.fontSizeSmall
                text: "View forecast →"
                color: Theme.primary
            }
        }
    }

    // Loading overlay
    Rectangle {
        anchors.fill: parent
        color: Theme.surface
        opacity: root.loading && !root.hasData ? 0.8 : 0
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
                font.pixelSize: 32
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

            Text {
                Layout.alignment: Qt.AlignHCenter
                font.pixelSize: Theme.fontSizeNormal
                text: "Loading weather..."
                color: Theme.textSecondary
            }
        }
    }
}
