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
    flags: Qt.Window | Qt.FramelessWindowHint

    property bool sidebarCollapsed: false
    property int sidebarExpandedWidth: 200
    property int sidebarCollapsedWidth: 56
    property string currentPage: "welcome"

    // Global weather model for sidebar and dashboard
    WeatherModel {
        id: weatherModel
        Component.onCompleted: refresh()
    }

    // Apply theme background
    color: Theme.background

    // Window drag and resize handling
    property point dragPosition

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Always-visible sidebar
        Rectangle {
            id: sidebar
            Layout.fillHeight: true
            Layout.preferredWidth: sidebarCollapsed ? sidebarCollapsedWidth : sidebarExpandedWidth
            color: Theme.sidebarBg

            Behavior on Layout.preferredWidth {
                NumberAnimation {
                    duration: 150
                    easing.type: Easing.OutQuad
                }
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingSm
                spacing: Theme.spacingXs

                // Logo area - clickable to go home, draggable for window
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    radius: Theme.buttonRadius
                    color: logoMouseArea.containsMouse ? Theme.sidebarHover : "transparent"

                    Behavior on color {
                        ColorAnimation {
                            duration: 100
                        }
                    }

                    MouseArea {
                        id: logoMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        property point clickPos
                        property bool isDragging: false

                        onPressed: mouse => {
                            clickPos = Qt.point(mouse.x, mouse.y);
                            isDragging = false;
                        }
                        onPositionChanged: mouse => {
                            if (pressed) {
                                const delta = Qt.point(mouse.x - clickPos.x, mouse.y - clickPos.y);
                                if (Math.abs(delta.x) > 5 || Math.abs(delta.y) > 5) {
                                    isDragging = true;
                                    root.x += delta.x;
                                    root.y += delta.y;
                                }
                            }
                        }
                        onReleased: {
                            if (!isDragging) {
                                currentPage = "welcome";
                                stackView.replace(null);
                            }
                        }
                        onDoubleClicked: {
                            if (root.visibility === Window.Maximized) {
                                root.showNormal();
                            } else {
                                root.showMaximized();
                            }
                        }
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacingXs
                        spacing: Theme.spacingSm

                        Rectangle {
                            width: 36
                            height: 36
                            radius: 8
                            color: Theme.primary

                            Label {
                                anchors.centerIn: parent
                                text: Icons.house
                                font.family: Icons.family
                                font.pixelSize: 18
                                color: Theme.primaryText
                            }
                        }

                        Label {
                            visible: !sidebarCollapsed
                            text: "MyMe"
                            font.pixelSize: Theme.fontSizeLarge
                            font.bold: true
                            color: Theme.text
                            Layout.fillWidth: true
                        }
                    }

                    ToolTip.visible: sidebarCollapsed && logoMouseArea.containsMouse
                    ToolTip.text: "Home"
                    ToolTip.delay: 500
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: Theme.sidebarBorder
                    Layout.topMargin: Theme.spacingXs
                    Layout.bottomMargin: Theme.spacingXs
                }

                // Navigation items
                Repeater {
                    model: [
                        {
                            id: "notes",
                            icon: Icons.notePencil,
                            label: "Notes",
                            enabled: true
                        },
                        {
                            id: "projects",
                            icon: Icons.squaresFour,
                            label: "Projects",
                            enabled: true
                        },
                        {
                            id: "repos",
                            icon: Icons.folderSimple,
                            label: "Repos",
                            enabled: true
                        },
                        {
                            id: "weather",
                            icon: Icons.sun,
                            label: "Weather",
                            enabled: true
                        },
                        {
                            id: "devtools",
                            icon: Icons.wrench,
                            label: "Dev Tools",
                            enabled: true
                        }
                    ]

                    delegate: Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 40
                        radius: Theme.buttonRadius
                        color: currentPage === modelData.id ? Theme.sidebarActive : navMouseArea.containsMouse ? Theme.sidebarHover : "transparent"
                        opacity: modelData.enabled ? 1.0 : 0.5

                        Behavior on color {
                            ColorAnimation {
                                duration: 100
                            }
                        }

                        MouseArea {
                            id: navMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: modelData.enabled ? Qt.PointingHandCursor : Qt.ForbiddenCursor
                            onClicked: {
                                if (modelData.enabled) {
                                    currentPage = modelData.id;
                                    if (modelData.id === "notes")
                                        stackView.replace("pages/NotePage.qml");
                                    else if (modelData.id === "projects")
                                        stackView.replace("pages/ProjectsPage.qml");
                                    else if (modelData.id === "repos")
                                        stackView.replace("pages/RepoPage.qml");
                                    else if (modelData.id === "weather")
                                        stackView.replace("pages/WeatherPage.qml");
                                    else if (modelData.id === "devtools")
                                        stackView.replace("pages/DevToolsPage.qml");
                                }
                            }
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Theme.spacingSm
                            anchors.rightMargin: Theme.spacingSm
                            spacing: Theme.spacingSm

                            // Active indicator
                            Rectangle {
                                width: 3
                                height: 20
                                radius: 2
                                color: currentPage === modelData.id ? Theme.primary : "transparent"
                            }

                            Label {
                                text: modelData.icon
                                font.family: Icons.family
                                font.pixelSize: 18
                                color: Theme.text
                                horizontalAlignment: Text.AlignHCenter
                                Layout.preferredWidth: 24
                            }

                            Label {
                                visible: !sidebarCollapsed
                                text: modelData.label
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.text
                                Layout.fillWidth: true
                            }
                        }

                        ToolTip.visible: sidebarCollapsed && navMouseArea.containsMouse
                        ToolTip.text: modelData.label
                        ToolTip.delay: 500
                    }
                }

                Item {
                    Layout.fillHeight: true
                }

                // Weather compact widget in sidebar footer
                WeatherCompact {
                    Layout.fillWidth: true
                    Layout.preferredHeight: sidebarCollapsed ? 40 : 48
                    expanded: !sidebarCollapsed
                    loading: weatherModel.loading
                    hasData: weatherModel.has_data
                    isStale: weatherModel.is_stale
                    temperature: weatherModel.temperature
                    condition: weatherModel.condition
                    conditionIcon: weatherModel.condition_icon

                    onClicked: {
                        currentPage = "weather";
                        stackView.replace("pages/WeatherPage.qml");
                    }

                    ToolTip.visible: sidebarCollapsed && !loading
                    ToolTip.text: hasData ? `${Math.round(temperature)}° ${condition}` : "Weather"
                    ToolTip.delay: 500
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: Theme.sidebarBorder
                    Layout.topMargin: Theme.spacingXs
                    Layout.bottomMargin: Theme.spacingXs
                }

                // Settings button
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 40
                    radius: Theme.buttonRadius
                    color: currentPage === "settings" ? Theme.sidebarActive : settingsMouseArea.containsMouse ? Theme.sidebarHover : "transparent"

                    Behavior on color {
                        ColorAnimation {
                            duration: 100
                        }
                    }

                    MouseArea {
                        id: settingsMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            currentPage = "settings";
                            stackView.replace("pages/SettingsPage.qml");
                        }
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.spacingSm
                        anchors.rightMargin: Theme.spacingSm
                        spacing: Theme.spacingSm

                        Rectangle {
                            width: 3
                            height: 20
                            radius: 2
                            color: currentPage === "settings" ? Theme.primary : "transparent"
                        }

                        Label {
                            text: Icons.gearSix
                            font.family: Icons.family
                            font.pixelSize: 18
                            color: Theme.text
                            horizontalAlignment: Text.AlignHCenter
                            Layout.preferredWidth: 24
                        }

                        Label {
                            visible: !sidebarCollapsed
                            text: "Settings"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            Layout.fillWidth: true
                        }
                    }

                    ToolTip.visible: sidebarCollapsed && settingsMouseArea.containsMouse
                    ToolTip.text: "Settings"
                    ToolTip.delay: 500
                }

                // Collapse toggle button
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 40
                    radius: Theme.buttonRadius
                    color: collapseMouseArea.containsMouse ? Theme.sidebarHover : "transparent"

                    Behavior on color {
                        ColorAnimation {
                            duration: 100
                        }
                    }

                    MouseArea {
                        id: collapseMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: sidebarCollapsed = !sidebarCollapsed
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.spacingSm
                        anchors.rightMargin: Theme.spacingSm
                        spacing: Theme.spacingSm

                        Rectangle {
                            width: 3
                            height: 20
                            radius: 2
                            color: "transparent"
                        }

                        Label {
                            text: sidebarCollapsed ? Icons.caretRight : Icons.caretLeft
                            font.family: Icons.family
                            font.pixelSize: 18
                            horizontalAlignment: Text.AlignHCenter
                            Layout.preferredWidth: 24
                            color: Theme.textSecondary
                        }

                        Label {
                            visible: !sidebarCollapsed
                            text: "Collapse"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textSecondary
                            Layout.fillWidth: true
                        }
                    }

                    ToolTip.visible: sidebarCollapsed && collapseMouseArea.containsMouse
                    ToolTip.text: "Expand sidebar"
                    ToolTip.delay: 500
                }
            }
        }

        // Separator line
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            color: Theme.border
        }

        // Main content area with window controls
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            // Window title bar / drag area
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 36
                color: Theme.background

                MouseArea {
                    anchors.fill: parent
                    property point clickPos

                    onPressed: mouse => {
                        clickPos = Qt.point(mouse.x, mouse.y);
                    }
                    onPositionChanged: mouse => {
                        if (pressed) {
                            const delta = Qt.point(mouse.x - clickPos.x, mouse.y - clickPos.y);
                            root.x += delta.x;
                            root.y += delta.y;
                        }
                    }
                    onDoubleClicked: {
                        if (root.visibility === Window.Maximized) {
                            root.showNormal();
                        } else {
                            root.showMaximized();
                        }
                    }
                }

                // Window controls
                RowLayout {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.rightMargin: Theme.spacingSm
                    spacing: 2

                    // Minimize button
                    Rectangle {
                        width: 32
                        height: 28
                        radius: 4
                        color: minimizeMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                        Label {
                            anchors.centerIn: parent
                            text: Icons.minus
                            font.family: Icons.family
                            font.pixelSize: 14
                            color: Theme.textSecondary
                        }

                        MouseArea {
                            id: minimizeMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.showMinimized()
                        }
                    }

                    // Maximize button
                    Rectangle {
                        width: 32
                        height: 28
                        radius: 4
                        color: maximizeMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                        Label {
                            anchors.centerIn: parent
                            text: root.visibility === Window.Maximized ? Icons.cornersIn : Icons.square
                            font.family: Icons.family
                            font.pixelSize: 14
                            color: Theme.textSecondary
                        }

                        MouseArea {
                            id: maximizeMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.visibility === Window.Maximized) {
                                    root.showNormal();
                                } else {
                                    root.showMaximized();
                                }
                            }
                        }
                    }

                    // Close button
                    Rectangle {
                        width: 32
                        height: 28
                        radius: 4
                        color: closeMouseArea.containsMouse ? Theme.error : "transparent"

                        Label {
                            anchors.centerIn: parent
                            text: Icons.x
                            font.family: Icons.family
                            font.pixelSize: 14
                            color: closeMouseArea.containsMouse ? "#ffffff" : Theme.textSecondary
                        }

                        MouseArea {
                            id: closeMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.close()
                        }
                    }
                }
            }

            StackView {
                id: stackView
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true

                replaceEnter: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 150
                    }
                }
                replaceExit: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 150
                    }
                }

                initialItem: Page {
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
                                    onClicked: {
                                        currentPage = "notes";
                                        stackView.replace("pages/NotePage.qml");
                                    }
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
                                    onClicked: {
                                        currentPage = "devtools";
                                        stackView.replace("pages/DevToolsPage.qml");
                                    }
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
                                    onClicked: {
                                        currentPage = "settings";
                                        stackView.replace("pages/SettingsPage.qml");
                                    }
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

                        // Weather widget dashboard card
                        WeatherWidget {
                            Layout.alignment: Qt.AlignHCenter
                            Layout.topMargin: Theme.spacingLg
                            loading: weatherModel.loading
                            hasData: weatherModel.has_data
                            isStale: weatherModel.is_stale
                            temperature: weatherModel.temperature
                            feelsLike: weatherModel.feels_like
                            humidity: weatherModel.humidity
                            windSpeed: weatherModel.wind_speed
                            condition: weatherModel.condition
                            conditionIcon: weatherModel.condition_icon
                            locationName: weatherModel.location_name
                            todayHigh: weatherModel.today_high
                            todayLow: weatherModel.today_low
                            precipChance: weatherModel.precipitation_chance
                            sunrise: weatherModel.sunrise
                            sunset: weatherModel.sunset

                            onClicked: {
                                currentPage = "weather";
                                stackView.replace("pages/WeatherPage.qml");
                            }

                            onRefreshRequested: weatherModel.refresh()
                        }
                    }
                }
            }
        }
    }
}
