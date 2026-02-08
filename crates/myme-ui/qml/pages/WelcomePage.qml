import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."
import "../components"

Page {
    id: welcomePage
    title: "Welcome"

    background: Rectangle {
        color: Theme.background
    }

    ColumnLayout {
        anchors.centerIn: parent
        spacing: Theme.spacingLg

        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 80
            height: 80
            radius: 16
            color: Theme.primary

            Label {
                anchors.centerIn: parent
                text: "M"
                font.pixelSize: 40
                font.bold: true
                color: Theme.primaryText
            }
        }

        Label {
            text: "Welcome to MyMe"
            font.pixelSize: Theme.fontSizeTitle
            font.bold: true
            color: Theme.text
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "Your Personal Productivity & Dev Hub"
            font.pixelSize: Theme.fontSizeMedium
            color: Theme.textSecondary
            Layout.alignment: Qt.AlignHCenter
        }

        Item {
            height: Theme.spacingMd
        }

        RowLayout {
            Layout.alignment: Qt.AlignHCenter
            spacing: Theme.spacingMd

            Rectangle {
                width: 140
                height: 100
                radius: Theme.cardRadius
                color: Theme.surface
                border.color: notesCardMouse.containsMouse ? Theme.primary : Theme.border
                border.width: 1

                Behavior on border.color {
                    ColorAnimation {
                        duration: 100
                    }
                }

                MouseArea {
                    id: notesCardMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("NotePage"))
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingSm

                    Label {
                        text: Icons.notePencil
                        font.family: Icons.family
                        font.pixelSize: 32
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Notes"
                        font.pixelSize: Theme.fontSizeNormal
                        font.bold: true
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }

            Rectangle {
                width: 140
                height: 100
                radius: Theme.cardRadius
                color: Theme.surface
                border.color: devtoolsCardMouse.containsMouse ? Theme.primary : Theme.border
                border.width: 1

                Behavior on border.color {
                    ColorAnimation {
                        duration: 100
                    }
                }

                MouseArea {
                    id: devtoolsCardMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("DevToolsPage"))
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingSm

                    Label {
                        text: Icons.wrench
                        font.family: Icons.family
                        font.pixelSize: 32
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Dev Tools"
                        font.pixelSize: Theme.fontSizeNormal
                        font.bold: true
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }

            Rectangle {
                width: 140
                height: 100
                radius: Theme.cardRadius
                color: Theme.surface
                border.color: settingsCardMouse.containsMouse ? Theme.primary : Theme.border
                border.width: 1

                Behavior on border.color {
                    ColorAnimation {
                        duration: 100
                    }
                }

                MouseArea {
                    id: settingsCardMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("SettingsPage"))
                }

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: Theme.spacingSm

                    Label {
                        text: Icons.gearSix
                        font.family: Icons.family
                        font.pixelSize: 32
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Settings"
                        font.pixelSize: Theme.fontSizeNormal
                        font.bold: true
                        color: Theme.text
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }

        // Google services widgets row
        RowLayout {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: Theme.spacingLg
            spacing: Theme.spacingMd

            EmailWidget {
                loading: AppContext.gmailModel ? AppContext.gmailModel.loading : false
                authenticated: AppContext.gmailModel ? AppContext.gmailModel.authenticated : false
                unreadCount: AppContext.gmailModel ? AppContext.gmailModel.unread_count : 0

                onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("GmailPage"))

                onRefreshRequested: {
                    if (AppContext.gmailModel) {
                        AppContext.gmailModel.fetch_messages()
                    }
                }
            }

            CalendarWidget {
                loading: AppContext.calendarModel ? AppContext.calendarModel.loading : false
                authenticated: AppContext.calendarModel ? AppContext.calendarModel.authenticated : false
                todayEventCount: AppContext.calendarModel ? AppContext.calendarModel.today_event_count : 0
                nextEventSummary: AppContext.calendarModel ? AppContext.calendarModel.next_event_summary : ""
                nextEventTime: AppContext.calendarModel ? AppContext.calendarModel.next_event_time : ""

                onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("CalendarPage"))

                onRefreshRequested: {
                    if (AppContext.calendarModel) {
                        AppContext.calendarModel.fetch_today_events()
                    }
                }
            }
        }

        // Weather widget dashboard card
        WeatherWidget {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: Theme.spacingMd
            loading: AppContext.weatherModel ? AppContext.weatherModel.loading : false
            hasData: AppContext.weatherModel ? AppContext.weatherModel.has_data : false
            isStale: AppContext.weatherModel ? AppContext.weatherModel.is_stale : false
            temperature: AppContext.weatherModel ? AppContext.weatherModel.temperature : 0
            feelsLike: AppContext.weatherModel ? AppContext.weatherModel.feels_like : 0
            humidity: AppContext.weatherModel ? AppContext.weatherModel.humidity : 0
            windSpeed: AppContext.weatherModel ? AppContext.weatherModel.wind_speed : 0
            condition: AppContext.weatherModel ? AppContext.weatherModel.condition : ""
            conditionIcon: AppContext.weatherModel ? AppContext.weatherModel.condition_icon : ""
            locationName: AppContext.weatherModel ? AppContext.weatherModel.location_name : ""
            todayHigh: AppContext.weatherModel ? AppContext.weatherModel.today_high : 0
            todayLow: AppContext.weatherModel ? AppContext.weatherModel.today_low : 0
            precipChance: AppContext.weatherModel ? AppContext.weatherModel.precipitation_chance : 0
            sunrise: AppContext.weatherModel ? AppContext.weatherModel.sunrise : ""
            sunset: AppContext.weatherModel ? AppContext.weatherModel.sunset : ""

            onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("WeatherPage"))

            onRefreshRequested: {
                if (AppContext.weatherModel) {
                    AppContext.weatherModel.refresh()
                }
            }
        }
    }
}
