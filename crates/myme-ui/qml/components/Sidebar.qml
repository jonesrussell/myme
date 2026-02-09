import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: sidebar

    property bool expanded: true
    property string currentPage: "WelcomePage"

    signal navigateTo(string pageName)

    color: Theme.sidebarBg
    width: expanded ? 220 : 60

    Behavior on width {
        NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
    }

    // Nav items model
    ListModel {
        id: navModel
        ListElement { title: "Dashboard"; page: "WelcomePage"; icon: "" }
        ListElement { title: "Notes"; page: "NotePage"; icon: "" }
        ListElement { title: "Gmail"; page: "GmailPage"; icon: "" }
        ListElement { title: "Calendar"; page: "CalendarPage"; icon: "" }
        ListElement { title: "Projects"; page: "ProjectsPage"; icon: "" }
        ListElement { title: "Repos"; page: "RepoPage"; icon: "" }
        ListElement { title: "Weather"; page: "WeatherPage"; icon: "" }
        ListElement { title: "Dev Tools"; page: "DevToolsPage"; icon: "" }
    }

    // Map page names to Phosphor icons
    function getNavIcon(page) {
        const iconMap = {
            "WelcomePage": Icons.house,
            "NotePage": Icons.notePencil,
            "GmailPage": Icons.envelopeSimple,
            "CalendarPage": Icons.calendarBlank,
            "ProjectsPage": Icons.squaresFour,
            "RepoPage": Icons.gitBranch,
            "WeatherPage": Icons.cloud_sun,
            "DevToolsPage": Icons.wrench,
            "SettingsPage": Icons.gearSix
        };
        return iconMap[page] || Icons.house;
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: Theme.spacingMd
        anchors.bottomMargin: Theme.spacingSm
        spacing: 0

        // Brand mark
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 48
            Layout.bottomMargin: Theme.spacingSm

            RowLayout {
                anchors.centerIn: parent
                spacing: Theme.spacingSm

                Rectangle {
                    width: 36
                    height: 36
                    radius: 10
                    color: Theme.primary

                    Label {
                        anchors.centerIn: parent
                        text: "M"
                        font.pixelSize: 20
                        font.weight: Font.Bold
                        font.family: Theme.fontFamily
                        color: Theme.primaryText
                    }
                }

                Label {
                    visible: sidebar.expanded
                    text: "MyMe"
                    font.pixelSize: Theme.fontSizeLarge
                    font.weight: Font.Bold
                    font.family: Theme.fontFamily
                    color: Theme.text

                    opacity: sidebar.expanded ? 1 : 0
                    Behavior on opacity {
                        NumberAnimation { duration: 150 }
                    }
                }
            }
        }

        // Separator
        Rectangle {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMd
            Layout.rightMargin: Theme.spacingMd
            Layout.bottomMargin: Theme.spacingSm
            height: 1
            color: Theme.borderLight
        }

        // Nav items
        Repeater {
            model: navModel

            delegate: Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 40
                Layout.leftMargin: Theme.spacingSm
                Layout.rightMargin: Theme.spacingSm

                property bool isActive: sidebar.currentPage === model.page
                property bool isHovered: navMouseArea.containsMouse

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.buttonRadius
                    color: isActive ? Theme.sidebarActive : (isHovered ? Theme.sidebarHover : "transparent")

                    Behavior on color {
                        ColorAnimation { duration: 100 }
                    }

                    // Active indicator bar
                    Rectangle {
                        visible: isActive
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        width: 3
                        height: 20
                        radius: 2
                        color: Theme.sidebarActiveIndicator
                    }

                    // Glow background for active item
                    Rectangle {
                        visible: isActive
                        anchors.fill: parent
                        radius: Theme.buttonRadius
                        color: Theme.primaryGlow
                        border.color: Qt.rgba(Theme.primary.r, Theme.primary.g, Theme.primary.b, 0.3)
                        border.width: 1
                        opacity: 0.3
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: sidebar.expanded ? Theme.spacingMd : 0
                        anchors.rightMargin: Theme.spacingSm
                        spacing: Theme.spacingSm

                        // Icon
                        Item {
                            Layout.preferredWidth: sidebar.expanded ? 24 : parent.width
                            Layout.preferredHeight: 24

                            Text {
                                anchors.centerIn: parent
                                font.family: Icons.family
                                font.pixelSize: 18
                                text: sidebar.getNavIcon(model.page)
                                color: isActive ? Theme.primary : (isHovered ? Theme.text : Theme.textSecondary)

                                Behavior on color {
                                    ColorAnimation { duration: 100 }
                                }
                            }
                        }

                        // Label
                        Label {
                            visible: sidebar.expanded
                            text: model.title
                            font.pixelSize: Theme.fontSizeNormal
                            font.family: Theme.fontFamily
                            font.weight: isActive ? Font.Medium : Font.Normal
                            color: isActive ? Theme.text : (isHovered ? Theme.text : Theme.textSecondary)
                            Layout.fillWidth: true
                            elide: Text.ElideRight

                            opacity: sidebar.expanded ? 1 : 0
                            Behavior on opacity {
                                NumberAnimation { duration: 150 }
                            }
                            Behavior on color {
                                ColorAnimation { duration: 100 }
                            }
                        }
                    }
                }

                MouseArea {
                    id: navMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: sidebar.navigateTo(model.page)
                }

                ToolTip {
                    visible: !sidebar.expanded && navMouseArea.containsMouse
                    text: model.title
                    delay: 500
                }
            }
        }

        // Spacer
        Item { Layout.fillHeight: true }

        // Weather compact (when expanded)
        WeatherCompact {
            visible: sidebar.expanded && AppContext.weatherModel
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingSm
            Layout.rightMargin: Theme.spacingSm
            Layout.bottomMargin: Theme.spacingSm
            expanded: sidebar.expanded
            loading: AppContext.weatherModel ? AppContext.weatherModel.loading : false
            hasData: AppContext.weatherModel ? AppContext.weatherModel.has_data : false
            isStale: AppContext.weatherModel ? AppContext.weatherModel.is_stale : false
            temperature: AppContext.weatherModel ? AppContext.weatherModel.temperature : 0
            condition: AppContext.weatherModel ? AppContext.weatherModel.condition : ""
            conditionIcon: AppContext.weatherModel ? AppContext.weatherModel.condition_icon : "sun"
            onClicked: sidebar.navigateTo("WeatherPage")
        }

        // Separator above bottom bar
        Rectangle {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacingMd
            Layout.rightMargin: Theme.spacingMd
            Layout.topMargin: Theme.spacingSm
            height: 1
            color: Theme.borderLight
        }

        // Bottom bar
        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 40
            Layout.leftMargin: Theme.spacingSm
            Layout.rightMargin: Theme.spacingSm
            Layout.topMargin: Theme.spacingSm
            spacing: Theme.spacingXs

            // Theme toggle
            Rectangle {
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                radius: Theme.buttonRadius
                color: themeMouseArea.containsMouse ? Theme.sidebarHover : "transparent"

                Text {
                    anchors.centerIn: parent
                    font.family: Icons.family
                    font.pixelSize: 16
                    text: Theme.mode === "dark" ? Icons.moon : (Theme.mode === "light" ? Icons.sun : Icons.circleHalf)
                    color: Theme.textSecondary
                }

                MouseArea {
                    id: themeMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (Theme.mode === "dark") Theme.mode = "light";
                        else if (Theme.mode === "light") Theme.mode = "auto";
                        else Theme.mode = "dark";
                    }
                }

                ToolTip {
                    visible: themeMouseArea.containsMouse
                    text: "Theme: " + Theme.mode
                    delay: 500
                }
            }

            // Settings
            Rectangle {
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                radius: Theme.buttonRadius
                color: sidebar.currentPage === "SettingsPage" ? Theme.sidebarActive : (settingsMouseArea.containsMouse ? Theme.sidebarHover : "transparent")

                Text {
                    anchors.centerIn: parent
                    font.family: Icons.family
                    font.pixelSize: 16
                    text: Icons.gearSix
                    color: sidebar.currentPage === "SettingsPage" ? Theme.primary : Theme.textSecondary
                }

                MouseArea {
                    id: settingsMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: sidebar.navigateTo("SettingsPage")
                }

                ToolTip {
                    visible: settingsMouseArea.containsMouse
                    text: "Settings"
                    delay: 500
                }
            }

            Item { Layout.fillWidth: true }

            // Collapse toggle
            Rectangle {
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                radius: Theme.buttonRadius
                color: collapseMouseArea.containsMouse ? Theme.sidebarHover : "transparent"

                Text {
                    anchors.centerIn: parent
                    font.family: Icons.family
                    font.pixelSize: 16
                    text: sidebar.expanded ? Icons.sidebarSimple : Icons.sidebar
                    color: Theme.textSecondary
                }

                MouseArea {
                    id: collapseMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: sidebar.expanded = !sidebar.expanded
                }

                ToolTip {
                    visible: collapseMouseArea.containsMouse
                    text: sidebar.expanded ? "Collapse sidebar" : "Expand sidebar"
                    delay: 500
                }
            }
        }
    }
}
