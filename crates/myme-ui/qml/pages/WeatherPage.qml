import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import myme_ui
import ".."

Kirigami.ScrollablePage {
    id: weatherPage
    title: "Weather Forecast"

    WeatherModel {
        id: weatherModel
        Component.onCompleted: refresh()
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

    actions: [
        Kirigami.Action {
            text: "Refresh"
            icon.name: "view-refresh"
            enabled: !weatherModel.loading
            onTriggered: weatherModel.refresh()
        }
    ]

    ColumnLayout {
        width: parent.width
        spacing: Kirigami.Units.largeSpacing

        // Error message
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: weatherModel.error_message !== ""
            type: Kirigami.MessageType.Warning
            text: weatherModel.error_message
        }

        // Stale data warning
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: weatherModel.is_stale && weatherModel.has_data
            type: Kirigami.MessageType.Information
            text: "Weather data may be outdated. Pull to refresh."
        }

        // Loading indicator
        Controls.BusyIndicator {
            Layout.alignment: Qt.AlignHCenter
            visible: weatherModel.loading && !weatherModel.has_data
            running: weatherModel.loading
        }

        // Current conditions card
        Kirigami.Card {
            Layout.fillWidth: true
            visible: weatherModel.has_data

            header: RowLayout {
                Kirigami.Heading {
                    text: weatherModel.location_name || "Current Location"
                    level: 3
                    Layout.fillWidth: true
                }
                Controls.Label {
                    text: weatherModel.is_stale ? "Cached" : "Live"
                    color: weatherModel.is_stale ?
                        Kirigami.Theme.neutralTextColor :
                        Kirigami.Theme.positiveTextColor
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            contentItem: ColumnLayout {
                spacing: Kirigami.Units.largeSpacing

                // Main temperature display
                RowLayout {
                    spacing: Kirigami.Units.largeSpacing

                    // Weather icon
                    Text {
                        font.family: Icons.family
                        font.pixelSize: 64
                        text: getWeatherIcon(weatherModel.condition_icon)
                        color: Kirigami.Theme.highlightColor
                    }

                    ColumnLayout {
                        spacing: 4

                        // Temperature
                        Controls.Label {
                            text: `${Math.round(weatherModel.temperature)}°`
                            font.pixelSize: 48
                            font.weight: Font.Bold
                        }

                        // Condition
                        Controls.Label {
                            text: weatherModel.condition
                            font.pixelSize: Theme.fontSizeLarge
                            color: Kirigami.Theme.disabledTextColor
                        }

                        // Feels like
                        Controls.Label {
                            text: `Feels like ${Math.round(weatherModel.feels_like)}°`
                            font.pixelSize: Theme.fontSizeNormal
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }

                    Item { Layout.fillWidth: true }

                    // High/Low column
                    ColumnLayout {
                        spacing: Kirigami.Units.smallSpacing

                        RowLayout {
                            spacing: 4
                            Text {
                                font.family: Icons.family
                                font.pixelSize: Theme.fontSizeNormal
                                text: Icons.caretUp
                                color: Kirigami.Theme.negativeTextColor
                            }
                            Controls.Label {
                                text: `${Math.round(weatherModel.today_high)}°`
                                font.pixelSize: Theme.fontSizeLarge
                            }
                        }

                        RowLayout {
                            spacing: 4
                            Text {
                                font.family: Icons.family
                                font.pixelSize: Theme.fontSizeNormal
                                text: Icons.caretDown
                                color: Kirigami.Theme.highlightColor
                            }
                            Controls.Label {
                                text: `${Math.round(weatherModel.today_low)}°`
                                font.pixelSize: Theme.fontSizeLarge
                            }
                        }
                    }
                }

                // Details grid
                GridLayout {
                    columns: 4
                    rowSpacing: Kirigami.Units.smallSpacing
                    columnSpacing: Kirigami.Units.largeSpacing

                    // Humidity
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        text: Icons.drop
                        color: Kirigami.Theme.highlightColor
                    }
                    Controls.Label {
                        text: `${weatherModel.humidity}%`
                        Layout.rightMargin: Kirigami.Units.largeSpacing
                    }

                    // Wind
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        text: Icons.wind
                        color: Kirigami.Theme.disabledTextColor
                    }
                    Controls.Label {
                        text: `${Math.round(weatherModel.wind_speed)} mph`
                    }

                    // Precipitation
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        text: Icons.cloud_rain
                        color: Kirigami.Theme.disabledTextColor
                    }
                    Controls.Label {
                        text: `${weatherModel.precipitation_chance}%`
                        Layout.rightMargin: Kirigami.Units.largeSpacing
                    }

                    // Sunrise
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        text: Icons.sun
                        color: Kirigami.Theme.neutralTextColor
                    }
                    Controls.Label {
                        text: weatherModel.sunrise
                    }

                    // Empty spacer
                    Item { width: 1 }
                    Item { width: 1 }

                    // Sunset
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeMedium
                        text: Icons.moon
                        color: Kirigami.Theme.disabledTextColor
                    }
                    Controls.Label {
                        text: weatherModel.sunset
                    }
                }
            }
        }

        // 7-Day Forecast
        Kirigami.Heading {
            text: "7-Day Forecast"
            level: 2
            visible: weatherModel.has_data
        }

        // Forecast cards
        Repeater {
            model: weatherModel.forecast_count()

            delegate: Kirigami.Card {
                Layout.fillWidth: true

                property int dayIndex: index
                property bool isToday: index === 0

                contentItem: RowLayout {
                    spacing: Kirigami.Units.largeSpacing

                    // Day name
                    Controls.Label {
                        text: isToday ? "Today" : weatherModel.get_forecast_day(dayIndex)
                        font.weight: isToday ? Font.Bold : Font.Normal
                        Layout.preferredWidth: 60
                    }

                    // Weather icon
                    Text {
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeLarge
                        text: getWeatherIcon(weatherModel.get_forecast_icon(dayIndex))
                        color: Kirigami.Theme.highlightColor
                    }

                    // Condition
                    Controls.Label {
                        text: weatherModel.get_forecast_condition(dayIndex)
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                        color: Kirigami.Theme.disabledTextColor
                    }

                    // Precipitation chance
                    RowLayout {
                        spacing: 4
                        visible: weatherModel.get_forecast_precip(dayIndex) > 0

                        Text {
                            font.family: Icons.family
                            font.pixelSize: Theme.fontSizeSmall
                            text: Icons.drop
                            color: Kirigami.Theme.highlightColor
                        }
                        Controls.Label {
                            text: `${weatherModel.get_forecast_precip(dayIndex)}%`
                            font.pixelSize: Theme.fontSizeSmall
                            color: Kirigami.Theme.disabledTextColor
                        }
                    }

                    // High temp
                    Controls.Label {
                        text: `${Math.round(weatherModel.get_forecast_high(dayIndex))}°`
                        font.weight: Font.Bold
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 40
                    }

                    // Low temp
                    Controls.Label {
                        text: `${Math.round(weatherModel.get_forecast_low(dayIndex))}°`
                        color: Kirigami.Theme.disabledTextColor
                        horizontalAlignment: Text.AlignRight
                        Layout.preferredWidth: 40
                    }
                }
            }
        }

        // Hourly forecast for today
        Kirigami.Heading {
            text: "Today's Hourly Forecast"
            level: 2
            visible: weatherModel.has_data && weatherModel.hourly_count(0) > 0
            Layout.topMargin: Kirigami.Units.largeSpacing
        }

        // Horizontal scrollable hourly forecast
        Controls.ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: 100
            visible: weatherModel.has_data && weatherModel.hourly_count(0) > 0
            Controls.ScrollBar.horizontal.policy: Controls.ScrollBar.AsNeeded
            Controls.ScrollBar.vertical.policy: Controls.ScrollBar.AlwaysOff

            RowLayout {
                spacing: Kirigami.Units.smallSpacing

                Repeater {
                    model: weatherModel.hourly_count(0)

                    delegate: Rectangle {
                        width: 60
                        height: 90
                        radius: Theme.cardRadius
                        color: Kirigami.Theme.backgroundColor
                        border.color: Kirigami.Theme.separatorColor
                        border.width: 1

                        property int hourIndex: index

                        ColumnLayout {
                            anchors.centerIn: parent
                            spacing: 4

                            // Time
                            Controls.Label {
                                text: weatherModel.get_hourly_time(0, hourIndex)
                                font.pixelSize: Theme.fontSizeSmall
                                Layout.alignment: Qt.AlignHCenter
                            }

                            // Icon
                            Text {
                                font.family: Icons.family
                                font.pixelSize: Theme.fontSizeMedium
                                text: getWeatherIcon(weatherModel.get_hourly_icon(0, hourIndex))
                                color: Kirigami.Theme.highlightColor
                                Layout.alignment: Qt.AlignHCenter
                            }

                            // Temperature
                            Controls.Label {
                                text: `${Math.round(weatherModel.get_hourly_temp(0, hourIndex))}°`
                                font.weight: Font.Medium
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
                                    color: Kirigami.Theme.highlightColor
                                }
                                Controls.Label {
                                    text: `${weatherModel.get_hourly_precip(0, hourIndex)}%`
                                    font.pixelSize: 10
                                    color: Kirigami.Theme.disabledTextColor
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
        Kirigami.PlaceholderMessage {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !weatherModel.has_data && !weatherModel.loading
            text: "Weather data unavailable"
            explanation: "Check your location settings and internet connection"
            icon.name: "weather-none-available"

            helpfulAction: Kirigami.Action {
                text: "Retry"
                icon.name: "view-refresh"
                onTriggered: weatherModel.refresh()
            }
        }
    }
}
