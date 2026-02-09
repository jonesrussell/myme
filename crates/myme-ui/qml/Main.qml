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
    minimumWidth: 480
    minimumHeight: 400
    visible: true
    title: "MyMe"
    color: Theme.background

    property string currentPage: "WelcomePage"

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

    // Navigate to a page by name
    function navigateToPage(pageName) {
        root.currentPage = pageName;
        AppContext.currentPage = pageName;
        AppContext.goToTopLevelPage(AppContext.pageUrl(pageName));
    }

    // Main layout: Sidebar + Content
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Persistent sidebar
        Sidebar {
            id: sidebarComponent
            Layout.fillHeight: true
            expanded: AppContext.sidebarExpanded
            currentPage: root.currentPage

            onExpandedChanged: AppContext.sidebarExpanded = expanded

            onNavigateTo: (pageName) => {
                root.navigateToPage(pageName);
            }
        }

        // Separator line
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            color: Theme.borderLight
        }

        // Page content
        StackView {
            id: stackView
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            pushEnter: Transition {
                ParallelAnimation {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 200
                        easing.type: Easing.OutCubic
                    }
                    PropertyAnimation {
                        property: "x"
                        from: 20
                        to: 0
                        duration: 200
                        easing.type: Easing.OutCubic
                    }
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
                ParallelAnimation {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 200
                        easing.type: Easing.OutCubic
                    }
                    PropertyAnimation {
                        property: "x"
                        from: -20
                        to: 0
                        duration: 200
                        easing.type: Easing.OutCubic
                    }
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
    }

    // Keyboard shortcuts for navigation
    Shortcut { sequence: "Ctrl+1"; onActivated: root.navigateToPage("WelcomePage") }
    Shortcut { sequence: "Ctrl+2"; onActivated: root.navigateToPage("NotePage") }
    Shortcut { sequence: "Ctrl+3"; onActivated: root.navigateToPage("GmailPage") }
    Shortcut { sequence: "Ctrl+4"; onActivated: root.navigateToPage("CalendarPage") }
    Shortcut { sequence: "Ctrl+5"; onActivated: root.navigateToPage("ProjectsPage") }
    Shortcut { sequence: "Ctrl+6"; onActivated: root.navigateToPage("RepoPage") }
    Shortcut { sequence: "Ctrl+7"; onActivated: root.navigateToPage("WeatherPage") }
    Shortcut { sequence: "Ctrl+8"; onActivated: root.navigateToPage("DevToolsPage") }
    Shortcut { sequence: "Ctrl+,"; onActivated: root.navigateToPage("SettingsPage") }
    Shortcut { sequence: "Ctrl+B"; onActivated: sidebarComponent.expanded = !sidebarComponent.expanded }

    Component.onCompleted: {
        AppContext.pageStack = stackView
        AppContext.weatherModel = weatherModel
        AppContext.gmailModel = gmailModel
        AppContext.calendarModel = calendarModel
        stackView.push(Qt.resolvedUrl("pages/WelcomePage.qml"))
    }
}
