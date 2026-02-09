import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."
import "../components"

Page {
    id: welcomePage
    title: "Dashboard"

    background: Rectangle {
        color: Theme.background
    }

    // Time-based greeting
    function getGreeting() {
        const hour = new Date().getHours();
        if (hour < 12) return "Good morning";
        if (hour < 17) return "Good afternoon";
        return "Good evening";
    }

    function getDateString() {
        const now = new Date();
        return Qt.formatDate(now, "dddd, MMMM d");
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        clip: true
        contentWidth: scroll.viewport.width

        ColumnLayout {
            width: scroll.viewport.width
            spacing: Theme.spacingLg

            // Greeting header
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.spacingXl
                Layout.rightMargin: Theme.spacingXl
                Layout.topMargin: Theme.spacingXl
                spacing: Theme.spacingXs

                Label {
                    text: getGreeting()
                    font.pixelSize: Theme.fontSizeTitle
                    font.weight: Font.Bold
                    font.family: Theme.fontFamily
                    color: Theme.text
                }

                Label {
                    text: getDateString()
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: Theme.fontFamily
                    color: Theme.textSecondary
                }
            }

            // Stat cards
            GridLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.spacingXl
                Layout.rightMargin: Theme.spacingXl
                columns: Responsive.columnsFor(scroll.viewport.width - Theme.spacingXl * 2, 200, 3)
                rowSpacing: Theme.spacingMd
                columnSpacing: Theme.spacingMd

                // Unread emails card
                Rectangle {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 160
                    Layout.preferredHeight: 90
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    border.width: 1

                    property bool hovered: emailStatMouse.containsMouse

                    scale: hovered ? 1.01 : 1.0
                    Behavior on scale { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

                    MouseArea {
                        id: emailStatMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("GmailPage"))
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        Text {
                            font.family: Icons.family
                            font.pixelSize: 28
                            text: Icons.envelopeSimple
                            color: Theme.primary
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Label {
                                text: AppContext.gmailModel ? (AppContext.gmailModel.authenticated ? AppContext.gmailModel.unread_count.toString() : "--") : "--"
                                font.pixelSize: Theme.fontSizeXLarge
                                font.weight: Font.Bold
                                font.family: Theme.fontFamily
                                color: Theme.text
                            }

                            Label {
                                text: "unread emails"
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamily
                                color: Theme.textSecondary
                            }
                        }
                    }
                }

                // Today's events card
                Rectangle {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 160
                    Layout.preferredHeight: 90
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    border.width: 1

                    property bool hovered: calStatMouse.containsMouse

                    scale: hovered ? 1.01 : 1.0
                    Behavior on scale { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

                    MouseArea {
                        id: calStatMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("CalendarPage"))
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        Text {
                            font.family: Icons.family
                            font.pixelSize: 28
                            text: Icons.calendarBlank
                            color: Theme.primary
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Label {
                                text: AppContext.calendarModel ? (AppContext.calendarModel.authenticated ? AppContext.calendarModel.today_event_count.toString() : "--") : "--"
                                font.pixelSize: Theme.fontSizeXLarge
                                font.weight: Font.Bold
                                font.family: Theme.fontFamily
                                color: Theme.text
                            }

                            Label {
                                text: "events today"
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamily
                                color: Theme.textSecondary
                            }
                        }
                    }
                }

                // Quick action card
                Rectangle {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 160
                    Layout.preferredHeight: 90
                    radius: Theme.cardRadius
                    color: Theme.surface
                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                    border.width: 1

                    property bool hovered: noteStatMouse.containsMouse

                    scale: hovered ? 1.01 : 1.0
                    Behavior on scale { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

                    MouseArea {
                        id: noteStatMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("NotePage"))
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingSm

                        Text {
                            font.family: Icons.family
                            font.pixelSize: 28
                            text: Icons.notePencil
                            color: Theme.primary
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2

                            Label {
                                text: "New note"
                                font.pixelSize: Theme.fontSizeMedium
                                font.weight: Font.Medium
                                font.family: Theme.fontFamily
                                color: Theme.text
                            }

                            Label {
                                text: "Quick capture"
                                font.pixelSize: Theme.fontSizeSmall
                                font.family: Theme.fontFamily
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }

            // Widget grid
            GridLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.spacingXl
                Layout.rightMargin: Theme.spacingXl
                columns: Responsive.columnsFor(scroll.viewport.width - Theme.spacingXl * 2, 300, 3)
                rowSpacing: Theme.spacingMd
                columnSpacing: Theme.spacingMd

                // Email widget
                EmailWidget {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 180
                    loading: AppContext.gmailModel ? AppContext.gmailModel.loading : false
                    authenticated: AppContext.gmailModel ? AppContext.gmailModel.authenticated : false
                    unreadCount: AppContext.gmailModel ? AppContext.gmailModel.unread_count : 0

                    onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("GmailPage"))
                    onRefreshRequested: {
                        if (AppContext.gmailModel) AppContext.gmailModel.fetch_messages();
                    }
                }

                // Calendar widget
                CalendarWidget {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 180
                    loading: AppContext.calendarModel ? AppContext.calendarModel.loading : false
                    authenticated: AppContext.calendarModel ? AppContext.calendarModel.authenticated : false
                    todayEventCount: AppContext.calendarModel ? AppContext.calendarModel.today_event_count : 0
                    nextEventSummary: AppContext.calendarModel ? AppContext.calendarModel.next_event_summary : ""
                    nextEventTime: AppContext.calendarModel ? AppContext.calendarModel.next_event_time : ""

                    onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl("CalendarPage"))
                    onRefreshRequested: {
                        if (AppContext.calendarModel) AppContext.calendarModel.fetch_today_events();
                    }
                }

                // Weather widget
                WeatherWidget {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 180
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
                        if (AppContext.weatherModel) AppContext.weatherModel.refresh();
                    }
                }
            }

            // Quick links section
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.spacingXl
                Layout.rightMargin: Theme.spacingXl
                Layout.bottomMargin: Theme.spacingXl
                spacing: Theme.spacingMd

                Label {
                    text: "Your workspace"
                    font.pixelSize: Theme.fontSizeMedium
                    font.weight: Font.Medium
                    font.family: Theme.fontFamily
                    color: Theme.text
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingMd

                    Repeater {
                        model: [
                            { title: "Projects", page: "ProjectsPage", icon: Icons.squaresFour },
                            { title: "Repos", page: "RepoPage", icon: Icons.gitBranch },
                            { title: "Dev Tools", page: "DevToolsPage", icon: Icons.wrench }
                        ]

                        delegate: Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 80
                            radius: Theme.cardRadius
                            color: quickLinkMouse.containsMouse ? Theme.surfaceHover : Theme.surface
                            border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                            border.width: 1

                            Behavior on color { ColorAnimation { duration: 100 } }

                            MouseArea {
                                id: quickLinkMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: AppContext.goToTopLevelPage(AppContext.pageUrl(modelData.page))
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingXs

                                Text {
                                    font.family: Icons.family
                                    font.pixelSize: 24
                                    text: modelData.icon
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    text: modelData.title
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.family: Theme.fontFamily
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
