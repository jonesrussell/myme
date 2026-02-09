pragma Singleton
import QtQuick

QtObject {
    id: appContext

    property var pageStack: null
    property var weatherModel: null
    property var gmailModel: null
    property var calendarModel: null

    property string currentPage: "WelcomePage"
    property bool sidebarExpanded: true

    function goToTopLevelPage(url) {
        if (pageStack) {
            pageStack.clear()
            pageStack.push(url)
        }
    }

    function pageUrl(name) {
        return Qt.resolvedUrl("pages/" + name + ".qml")
    }
}
