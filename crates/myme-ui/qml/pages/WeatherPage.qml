import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: weatherPage
    title: "Weather"

    background: Rectangle {
        color: Theme.background
    }

    WeatherModel {
        id: weatherModel
        Component.onCompleted: refresh()
    }

    // Timer to poll for async weather operation results
    Timer {
        id: weatherPollTimer
        interval: 100
        running: weatherModel.loading
        repeat: true
        onTriggered: weatherModel.poll_channel()
    }

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

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Weather"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }

            // Refresh button
            Rectangle {
                width: 36
                height: 36
                radius: Theme.buttonRadius
                color: refreshMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                Label {
                    anchors.centerIn: parent
                    text: Icons.arrowsClockwise
                    font.family: Icons.family
                    font.pixelSize: 18
                    color: Theme.text
                }

                MouseArea {
                    id: refreshMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    enabled: !weatherModel.loading
                    onClicked: weatherModel.refresh()
                }

                ToolTip.visible: refreshMouseArea.containsMouse
                ToolTip.text: "Refresh weather"
                ToolTip.delay: 500
            }

            Item { width: Theme.spacingMd }
        }
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingLg

            // Error message
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: errorContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.error + "20"
                border.color: Theme.error
                border.width: 1
                radius: Theme.cardRadius
                visible: weatherModel.error_message !== ""

                RowLayout {
                    id: errorContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingSm

                    Text {
                        text: Icons.warning
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.error
                    }

                    Label {
                        text: weatherModel.error_message
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.text
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
            }

            // Stale data warning
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: staleContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.warning + "20"
                border.color: Theme.warning
                border.width: 1
                radius: Theme.cardRadius
                visible: weatherModel.is_stale && weatherModel.has_data

                RowLayout {
                    id: staleContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingSm

                    Text {
                        text: Icons.info
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        color: Theme.warning
                    }

                    Label {
                        text: "Weather data may be outdated. Click refresh to update."
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.text
                        Layout.fillWidth: true
                    }
                }
            }

            // Loading indicator
            BusyIndicator {
                Layout.alignment: Qt.AlignHCenter
                visible: weatherModel.loading && !weatherModel.has_data
                running: weatherModel.loading
            }

            // Current conditions card
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: currentContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius
                visible: weatherModel.has_data

                ColumnLayout {
                    id: currentContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    // Header
                    RowLayout {
                        Layout.fillWidth: true

                        Label {
                            text: weatherModel.location_name || "Current Location"
                            font.pixelSize: Theme.fontSizeMedium
                            font.bold: true
                            color: Theme.text
                            Layout.fillWidth: true
                        }

                        Rectangle {
                            width: statusLabel.implicitWidth + Theme.spacingSm * 2
                            height: statusLabel.implicitHeight + Theme.spacingXs * 2
                            radius: Theme.buttonRadius
                            color: weatherModel.is_stale ? Theme.warning + "30" : Theme.success + "30"

                            Label {
                                id: statusLabel
                                anchors.centerIn: parent
                                text: weatherModel.is_stale ? "Cached" : "Live"
                                font.pixelSize: Theme.fontSizeSmall
                                color: weatherModel.is_stale ? Theme.warning : Theme.success
                            }
                        }
                    }

                    // Main temperature display
                    RowLayout {
                        spacing: Theme.spacingLg

                        // Weather icon
                        Text {
                            font.family: Icons.family
                            font.pixelSize: 64
                            text: getWeatherIcon(weatherModel.condition_icon)
                            color: Theme.primary
                        }

                        ColumnLayout {
                            spacing: 4

                            // Temperature
                            Label {
                                text: `${Math.round(weatherModel.temperature)}°`
                                font.pixelSize: 48
                                font.weight: Font.Bold
                                color: Theme.text
                            }

                            // Condition
                            Label {
                                text: weatherModel.condition
                                font.pixelSize: Theme.fontSizeLarge
                                color: Theme.textSecondary
                            }

                            // Feels like
                            Label {
                                text: `Feels like ${Math.round(weatherModel.feels_like)}°`
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textMuted
                            }
                        }

                        Item { Layout.fillWidth: true }

                        // High/Low column
                        ColumnLayout {
                            spacing: Theme.spacingSm

                            RowLayout {
                                spacing: 4
                                Text {
                                    font.family: Icons.family
                                    font.pixelSize: Theme.fontSizeNormal
                                    text: Icons.caretUp
                                    color: Theme.error
                                }
                                Label {
                                    text: `${Math.round(weatherModel.today_high)}°`
                                    font.pixelSize: Theme.fontSizeLarge
                                    color: Theme.text
                                }
                            }

                            RowLayout {
                                spacing: 4
                                Text {
                                    font.family: Icons.family
                                    font.pixelSize: Theme.fontSizeNormal
                                    text: Icons.caretDown
                                    color: Theme.primary
                                }
                                Label {
                                    text: `${Math.round(weatherModel.today_low)}°`
                                    font.pixelSize: Theme.fontSizeLarge
                                    color: Theme.text
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Theme.border
                    }

                    // Details grid
                    GridLayout {
                        columns: 4
                        rowSpacing: Theme.spacingSm
                        columnSpacing: Theme.spacingLg

                        // Humidity
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeMedium
                            text: Icons.drop
                            color: Theme.primary
                        }
                        Label {
                            text: `${weatherModel.humidity}%`
                            color: Theme.text
                            Layout.rightMargin: Theme.spacingLg
                        }

                        // Wind
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeMedium
                            text: Icons.wind
                            color: Theme.textSecondary
                        }
                        Label {
                            text: `${Math.round(weatherModel.wind_speed)} mph`
                            color: Theme.text
                        }

                        // Precipitation
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeMedium
                            text: Icons.cloud_rain
                            color: Theme.textSecondary
                        }
                        Label {
                            text: `${weatherModel.precipitation_chance}%`
                            color: Theme.text
                            Layout.rightMargin: Theme.spacingLg
                        }

                        // Sunrise
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeMedium
                            text: Icons.sun
                            color: Theme.warning
                        }
                        Label {
                            text: weatherModel.sunrise
                            color: Theme.text
                        }

                        // Empty spacer
                        Item { width: 1 }
                        Item { width: 1 }

                        // Sunset
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeMedium
                            text: Icons.moon
                            color: Theme.textMuted
                        }
                        Label {
                            text: weatherModel.sunset
                            color: Theme.text
                        }
                    }
                }
            }

            // 7-Day Forecast heading
            Label {
                text: "7-Day Forecast"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
                visible: weatherModel.has_data
                Layout.topMargin: Theme.spacingSm
            }

            // Forecast cards
            Repeater {
                model: weatherModel.forecast_count()

                delegate: Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    color: Theme.surface
                    border.color: Theme.border
                    border.width: 1
                    radius: Theme.cardRadius

                    property int dayIndex: index
                    property bool isToday: index === 0

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingMd

                        // Day name
                        Label {
                            text: isToday ? "Today" : weatherModel.get_forecast_day(dayIndex)
                            font.weight: isToday ? Font.Bold : Font.Normal
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            Layout.preferredWidth: 60
                        }

                        // Weather icon
                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeLarge
                            text: getWeatherIcon(weatherModel.get_forecast_icon(dayIndex))
                            color: Theme.primary
                        }

                        // Condition
                        Label {
                            text: weatherModel.get_forecast_condition(dayIndex)
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textSecondary
                            Layout.fillWidth: true
                            elide: Text.ElideRight
                        }

                        // Precipitation chance
                        RowLayout {
                            spacing: 4
                            visible: weatherModel.get_forecast_precip(dayIndex) > 0

                            Text {
                                font.family: Icons.family
                                font.pixelSize: Theme.fontSizeSmall
                                text: Icons.drop
                                color: Theme.primary
                            }
                            Label {
                                text: `${weatherModel.get_forecast_precip(dayIndex)}%`
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }

                        // High temp
                        Label {
                            text: `${Math.round(weatherModel.get_forecast_high(dayIndex))}°`
                            font.weight: Font.Bold
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            horizontalAlignment: Text.AlignRight
                            Layout.preferredWidth: 40
                        }

                        // Low temp
                        Label {
                            text: `${Math.round(weatherModel.get_forecast_low(dayIndex))}°`
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textMuted
                            horizontalAlignment: Text.AlignRight
                            Layout.preferredWidth: 40
                        }
                    }
                }
            }

            // Hourly forecast heading
            Label {
                text: "Today's Hourly Forecast"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
                visible: weatherModel.has_data && weatherModel.hourly_count(0) > 0
                Layout.topMargin: Theme.spacingMd
            }

            // Horizontal scrollable hourly forecast
            ScrollView {
                Layout.fillWidth: true
                Layout.preferredHeight: 100
                visible: weatherModel.has_data && weatherModel.hourly_count(0) > 0
                ScrollBar.horizontal.policy: ScrollBar.AsNeeded
                ScrollBar.vertical.policy: ScrollBar.AlwaysOff

                RowLayout {
                    spacing: Theme.spacingSm

                    Repeater {
                        model: weatherModel.hourly_count(0)

                        delegate: Rectangle {
                            width: 60
                            height: 90
                            radius: Theme.cardRadius
                            color: Theme.surface
                            border.color: Theme.border
                            border.width: 1

                            property int hourIndex: index

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: 4

                                // Time
                                Label {
                                    text: weatherModel.get_hourly_time(0, hourIndex)
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                // Icon
                                Text {
                                    font.family: Icons.family
                                    font.pixelSize: Theme.fontSizeMedium
                                    text: getWeatherIcon(weatherModel.get_hourly_icon(0, hourIndex))
                                    color: Theme.primary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                // Temperature
                                Label {
                                    text: `${Math.round(weatherModel.get_hourly_temp(0, hourIndex))}°`
                                    font.weight: Font.Medium
                                    font.pixelSize: Theme.fontSizeNormal
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                // Precipitation
                                RowLayout {
                                    Layout.alignment: Qt.AlignHCenter
                                    spacing: 2
                                    visible: weatherModel.get_hourly_precip(0, hourIndex) > 0

                                    Text {
                                        font.family: Icons.family
                                        font.pixelSize: 10
                                        text: Icons.drop
                                        color: Theme.primary
                                    }
                                    Label {
                                        text: `${weatherModel.get_hourly_precip(0, hourIndex)}%`
                                        font.pixelSize: 10
                                        color: Theme.textMuted
                                    }
                                }

                                // Spacer if no precip
                                Item {
                                    height: 14
                                    visible: weatherModel.get_hourly_precip(0, hourIndex) === 0
                                }
                            }
                        }
                    }
                }
            }

            // Placeholder when no data
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 200
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius
                visible: !weatherModel.has_data && !weatherModel.loading

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingMd

                    Text {
                        text: Icons.cloud
                        font.family: Icons.family
                        font.pixelSize: 48
                        color: Theme.textMuted
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Weather data unavailable"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Check your location settings and internet connection"
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Rectangle {
                        Layout.alignment: Qt.AlignHCenter
                        Layout.topMargin: Theme.spacingSm
                        width: retryLabel.implicitWidth + Theme.spacingMd * 2
                        height: retryLabel.implicitHeight + Theme.spacingSm * 2
                        radius: Theme.buttonRadius
                        color: retryMouseArea.containsMouse ? Theme.primaryHover : Theme.primary

                        Label {
                            id: retryLabel
                            anchors.centerIn: parent
                            text: "Retry"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.primaryText
                        }

                        MouseArea {
                            id: retryMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: weatherModel.refresh()
                        }
                    }
                }
            }

            Item {
                Layout.fillHeight: true
            }
        }
    }
}
