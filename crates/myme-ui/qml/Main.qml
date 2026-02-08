import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import myme_ui
import "."
import "components"

Kirigami.ApplicationWindow {
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

    globalDrawer: Kirigami.GlobalDrawer {
        title: "MyMe"
        titleIcon: "applications-utilities"

        actions: [
            Kirigami.Action {
                text: "Home"
                icon.name: "go-home"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("WelcomePage"))
            },
            Kirigami.Action {
                text: "Notes"
                icon.name: "view-task"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("NotePage"))
            },
            Kirigami.Action {
                text: "Gmail"
                icon.name: "mail-message"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("GmailPage"))
            },
            Kirigami.Action {
                text: "Calendar"
                icon.name: "view-calendar"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("CalendarPage"))
            },
            Kirigami.Action {
                text: "Projects"
                icon.name: "project-development"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("ProjectsPage"))
            },
            Kirigami.Action {
                text: "Repos"
                icon.name: "folder-git"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("RepoPage"))
            },
            Kirigami.Action {
                text: "Weather"
                icon.name: "weather-clear"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("WeatherPage"))
            },
            Kirigami.Action {
                text: "Dev Tools"
                icon.name: "utilities-development"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("DevToolsPage"))
            },
            Kirigami.Action {
                text: "Settings"
                icon.name: "configure"
                onTriggered: AppContext.goToTopLevelPage(AppContext.pageUrl("SettingsPage"))
            }
        ]
    }

    pageStack.initialPage: Qt.resolvedUrl("pages/WelcomePage.qml")

    Component.onCompleted: {
        AppContext.pageStack = root.pageStack
        AppContext.weatherModel = weatherModel
        AppContext.gmailModel = gmailModel
        AppContext.calendarModel = calendarModel
    }
}
