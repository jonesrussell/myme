import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import "."
import "components"

ApplicationWindow {
    id: root
    width: 1200
    height: 800
    visible: true
    title: "MyMe"
    color: Theme.background

    // Global weather model for dashboard
    WeatherModel {
        id: weatherModel
        Component.onCompleted: refresh()
    }

    Timer {
        id: weatherPollTimer
        interval: 100
        running: weatherModel.loading
        repeat: true
        onTriggered: weatherModel.poll_channel()
    }

    // Global Gmail model for dashboard
    GmailModel {
        id: gmailModel
        Component.onCompleted: {
            check_auth()
            if (authenticated) {
                fetch_messages()
            }
        }
    }

    Timer {
        id: gmailPollTimer
        interval: 100
        running: gmailModel.loading
        repeat: true
        onTriggered: gmailModel.poll_channel()
    }

    // Global Calendar model for dashboard
    CalendarModel {
        id: calendarModel
        Component.onCompleted: {
            check_auth()
            if (authenticated) {
                fetch_today_events()
            }
        }
    }

    Timer {
        id: calendarPollTimer
        interval: 100
        running: calendarModel.loading
        repeat: true
        onTriggered: calendarModel.poll_channel()
    }

    // Navigation drawer (hamburger menu)
    Drawer {
        id: navDrawer
        width: Math.min(280, root.width * 0.8)
        height: root.height
        interactive: true
        edge: Qt.LeftEdge
        dim: true
        modal: true
        background: Rectangle {
            color: Theme.sidebarBg
        }
        contentItem: ListView {
            clip: true
            header: Item {
                width: navDrawer.width - 20
                height: 56
                Label {
                    anchors.centerIn: parent
                    text: "MyMe"
                    font.pixelSize: Theme.fontSizeLarge
                    font.bold: true
                    color: Theme.text
                }
            }
            model: ListModel {
                ListElement { title: "Home"; page: "WelcomePage" }
                ListElement { title: "Notes"; page: "NotePage" }
                ListElement { title: "Gmail"; page: "GmailPage" }
                ListElement { title: "Calendar"; page: "CalendarPage" }
                ListElement { title: "Projects"; page: "ProjectsPage" }
                ListElement { title: "Repos"; page: "RepoPage" }
                ListElement { title: "Weather"; page: "WeatherPage" }
                ListElement { title: "Dev Tools"; page: "DevToolsPage" }
                ListElement { title: "Settings"; page: "SettingsPage" }
            }
            delegate: ItemDelegate {
                width: navDrawer.width - 20
                text: model.title
                font.pixelSize: Theme.fontSizeNormal
                onClicked: {
                    AppContext.goToTopLevelPage(AppContext.pageUrl(model.page))
                    navDrawer.close()
                }
            }
        }
    }

    // Toolbar with hamburger button
    header: ToolBar {
        id: toolBar
        background: Rectangle { color: Theme.surface }
        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingSm
            ToolButton {
                icon.source: ""
                text: "\u2630"
                font.pixelSize: 20
                onClicked: navDrawer.open()
            }
            Label {
                text: root.title
                font.pixelSize: Theme.fontSizeMedium
                color: Theme.text
                Layout.fillWidth: true
            }
        }
    }

    // Page stack (replaces Kirigami pageStack)
    StackView {
        id: stackView
        anchors.fill: parent
        clip: true

        pushEnter: Transition {
            PropertyAnimation {
                property: "opacity"
                from: 0
                to: 1
                duration: 150
            }
        }
        pushExit: Transition {
            PropertyAnimation {
                property: "opacity"
                from: 1
                to: 0
                duration: 150
            }
        }
        popEnter: Transition {
            PropertyAnimation {
                property: "opacity"
                from: 0
                to: 1
                duration: 150
            }
        }
        popExit: Transition {
            PropertyAnimation {
                property: "opacity"
                from: 1
                to: 0
                duration: 150
            }
        }
    }

    Component.onCompleted: {
        AppContext.pageStack = stackView
        AppContext.weatherModel = weatherModel
        AppContext.gmailModel = gmailModel
        AppContext.calendarModel = calendarModel
        stackView.push(Qt.resolvedUrl("pages/WelcomePage.qml"))
    }
}
