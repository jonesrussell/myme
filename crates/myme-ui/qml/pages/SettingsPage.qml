import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: settingsPage
    title: "Settings"

    // Auth model for connected accounts (GitHub)
    AuthModel {
        id: authModel
        Component.onCompleted: authModel.check_auth()
    }

    // Google Auth model
    GoogleAuthModel {
        id: googleAuthModel
        Component.onCompleted: googleAuthModel.check_auth()
    }

    // Timer to poll for async auth operation results
    Timer {
        id: authPollTimer
        interval: 100
        running: authModel.loading || googleAuthModel.loading
        repeat: true
        onTriggered: {
            authModel.poll_channel()
            googleAuthModel.poll_channel()
        }
    }

    background: Rectangle {
        color: Theme.background
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Settings"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }
        }
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        clip: true

        ColumnLayout {
            width: parent.width
            spacing: Theme.spacingLg

            // Appearance Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: appearanceContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: appearanceContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: "Appearance"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                    }

                    Label {
                        text: "Choose how MyMe looks to you. Select a single theme, or sync with your system settings."
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Theme selection
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        spacing: Theme.spacingMd

                        // Light theme option
                        Rectangle {
                            Layout.preferredWidth: 140
                            Layout.preferredHeight: 100
                            radius: Theme.cardRadius
                            color: Theme.mode === "light" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: Theme.mode === "light" ? Theme.primary : Theme.border
                            border.width: Theme.mode === "light" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: Theme.mode = "light"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingSm

                                Rectangle {
                                    Layout.alignment: Qt.AlignHCenter
                                    width: 40
                                    height: 40
                                    radius: 20
                                    color: "#f8f9fa"
                                    border.color: "#dee2e6"
                                    border.width: 1

                                    Text {
                                        anchors.centerIn: parent
                                        text: Icons.sun
                                        font.family: Icons.family
                                        font.pixelSize: 20
                                        color: "#f59e0b"
                                    }
                                }

                                Label {
                                    text: "Light"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: Theme.mode === "light"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        // Dark theme option
                        Rectangle {
                            Layout.preferredWidth: 140
                            Layout.preferredHeight: 100
                            radius: Theme.cardRadius
                            color: Theme.mode === "dark" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: Theme.mode === "dark" ? Theme.primary : Theme.border
                            border.width: Theme.mode === "dark" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: Theme.mode = "dark"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingSm

                                Rectangle {
                                    Layout.alignment: Qt.AlignHCenter
                                    width: 40
                                    height: 40
                                    radius: 20
                                    color: "#1a1a2e"
                                    border.color: "#2d3a5c"
                                    border.width: 1

                                    Text {
                                        anchors.centerIn: parent
                                        text: Icons.moon
                                        font.family: Icons.family
                                        font.pixelSize: 20
                                        color: "#a5b4fc"
                                    }
                                }

                                Label {
                                    text: "Dark"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: Theme.mode === "dark"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        // Auto theme option
                        Rectangle {
                            Layout.preferredWidth: 140
                            Layout.preferredHeight: 100
                            radius: Theme.cardRadius
                            color: Theme.mode === "auto" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: Theme.mode === "auto" ? Theme.primary : Theme.border
                            border.width: Theme.mode === "auto" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: Theme.mode = "auto"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingSm

                                Rectangle {
                                    Layout.alignment: Qt.AlignHCenter
                                    width: 40
                                    height: 40
                                    radius: 20
                                    gradient: Gradient {
                                        orientation: Gradient.Horizontal
                                        GradientStop {
                                            position: 0.0
                                            color: "#f8f9fa"
                                        }
                                        GradientStop {
                                            position: 1.0
                                            color: "#1a1a2e"
                                        }
                                    }
                                    border.color: "#888"
                                    border.width: 1

                                    Text {
                                        anchors.centerIn: parent
                                        text: Icons.circleHalf
                                        font.family: Icons.family
                                        font.pixelSize: 20
                                        color: "#888"
                                    }
                                }

                                Label {
                                    text: "Auto"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: Theme.mode === "auto"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                        }
                    }

                    // Current theme indicator
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        height: 40
                        radius: Theme.inputRadius
                        color: Theme.surfaceAlt

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingSm

                            Label {
                                text: Theme.mode === "auto" ? "Currently using " + (Theme.isDark ? "dark" : "light") + " theme (synced with system)" : "Using " + Theme.mode + " theme"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }

            // Weather Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: weatherContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                property string temperatureUnit: "auto"

                ColumnLayout {
                    id: weatherContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: "Weather"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                    }

                    Label {
                        text: "Configure how weather information is displayed."
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Temperature unit label
                    Label {
                        text: "Temperature Unit"
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.text
                        Layout.topMargin: Theme.spacingSm
                    }

                    // Temperature unit selection
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingMd

                        // Auto option
                        Rectangle {
                            Layout.preferredWidth: 100
                            Layout.preferredHeight: 70
                            radius: Theme.cardRadius
                            color: parent.parent.parent.temperatureUnit === "auto" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: parent.parent.parent.temperatureUnit === "auto" ? Theme.primary : Theme.border
                            border.width: parent.parent.parent.temperatureUnit === "auto" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: parent.parent.parent.parent.temperatureUnit = "auto"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingXs

                                Text {
                                    text: Icons.circleHalf
                                    font.family: Icons.family
                                    font.pixelSize: 20
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    text: "Auto"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: parent.parent.parent.parent.parent.temperatureUnit === "auto"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        // Celsius option
                        Rectangle {
                            Layout.preferredWidth: 100
                            Layout.preferredHeight: 70
                            radius: Theme.cardRadius
                            color: parent.parent.parent.temperatureUnit === "celsius" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: parent.parent.parent.temperatureUnit === "celsius" ? Theme.primary : Theme.border
                            border.width: parent.parent.parent.temperatureUnit === "celsius" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: parent.parent.parent.parent.temperatureUnit = "celsius"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingXs

                                Label {
                                    text: "°C"
                                    font.pixelSize: 20
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    text: "Celsius"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: parent.parent.parent.parent.parent.temperatureUnit === "celsius"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        // Fahrenheit option
                        Rectangle {
                            Layout.preferredWidth: 100
                            Layout.preferredHeight: 70
                            radius: Theme.cardRadius
                            color: parent.parent.parent.temperatureUnit === "fahrenheit" ? Theme.primary + "20" : Theme.surfaceAlt
                            border.color: parent.parent.parent.temperatureUnit === "fahrenheit" ? Theme.primary : Theme.border
                            border.width: parent.parent.parent.temperatureUnit === "fahrenheit" ? 2 : 1

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: parent.parent.parent.parent.temperatureUnit = "fahrenheit"
                            }

                            ColumnLayout {
                                anchors.centerIn: parent
                                spacing: Theme.spacingXs

                                Label {
                                    text: "°F"
                                    font.pixelSize: 20
                                    font.bold: true
                                    color: Theme.textSecondary
                                    Layout.alignment: Qt.AlignHCenter
                                }

                                Label {
                                    text: "Fahrenheit"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: parent.parent.parent.parent.parent.temperatureUnit === "fahrenheit"
                                    color: Theme.text
                                    Layout.alignment: Qt.AlignHCenter
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                        }
                    }

                    // Info text
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        height: 40
                        radius: Theme.inputRadius
                        color: Theme.surfaceAlt

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingSm

                            Text {
                                text: Icons.info
                                font.family: Icons.family
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textMuted
                            }

                            Label {
                                text: "Auto detects your preferred unit from system locale"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }

            // Connected Accounts Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: accountsContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: accountsContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: "Connected Accounts"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                    }

                    Label {
                        text: "Connect your accounts to enable additional features like project management and repository access."
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // GitHub Account
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        height: 72
                        radius: Theme.cardRadius
                        color: Theme.surfaceAlt
                        border.color: authModel.authenticated ? Theme.success : Theme.border
                        border.width: authModel.authenticated ? 2 : 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMd
                            spacing: Theme.spacingMd

                            // GitHub icon
                            Rectangle {
                                width: 44
                                height: 44
                                radius: 22
                                color: Theme.isDark ? "#333" : "#24292f"

                                Text {
                                    anchors.centerIn: parent
                                    text: Icons.githubLogo
                                    font.family: Icons.family
                                    font.pixelSize: 24
                                    color: "#ffffff"
                                }
                            }

                            // Account info
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                Label {
                                    text: "GitHub"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: true
                                    color: Theme.text
                                }

                                Label {
                                    text: authModel.authenticated ? "Connected" : "Not connected"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: authModel.authenticated ? Theme.success : Theme.textSecondary
                                }
                            }

                            // Connect/Disconnect button
                            Button {
                                text: authModel.loading ? "Connecting..." : (authModel.authenticated ? "Disconnect" : "Connect")
                                enabled: !authModel.loading
                                Layout.preferredWidth: 110
                                Layout.preferredHeight: 36

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: {
                                        if (!parent.enabled) return Theme.surfaceAlt
                                        if (authModel.authenticated) return Theme.error + "20"
                                        return parent.hovered ? Theme.primaryHover : Theme.primary
                                    }
                                    border.color: authModel.authenticated ? Theme.error : "transparent"
                                    border.width: authModel.authenticated ? 1 : 0
                                }

                                contentItem: Label {
                                    text: parent.text
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: true
                                    color: {
                                        if (!parent.enabled) return Theme.textMuted
                                        if (authModel.authenticated) return Theme.error
                                        return Theme.primaryText
                                    }
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }

                                onClicked: {
                                    if (authModel.authenticated) {
                                        authModel.sign_out()
                                    } else {
                                        authModel.authenticate()
                                    }
                                }
                            }
                        }
                    }

                    // GitHub Error message
                    Label {
                        visible: authModel.error_message !== ""
                        text: authModel.error_message
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.error
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Google Account
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingMd
                        height: 72
                        radius: Theme.cardRadius
                        color: Theme.surfaceAlt
                        border.color: googleAuthModel.authenticated ? Theme.success : Theme.border
                        border.width: googleAuthModel.authenticated ? 2 : 1

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMd
                            spacing: Theme.spacingMd

                            // Google icon
                            Rectangle {
                                width: 44
                                height: 44
                                radius: 22
                                color: "#ffffff"
                                border.color: Theme.border
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: "G"
                                    font.pixelSize: 24
                                    font.bold: true
                                    color: "#4285F4"
                                }
                            }

                            // Account info
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                Label {
                                    text: "Google"
                                    font.pixelSize: Theme.fontSizeNormal
                                    font.bold: true
                                    color: Theme.text
                                }

                                Label {
                                    text: googleAuthModel.authenticated ? (googleAuthModel.user_email || "Connected") : "Not connected"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: googleAuthModel.authenticated ? Theme.success : Theme.textSecondary
                                }
                            }

                            // Connect/Disconnect button
                            Button {
                                text: googleAuthModel.loading ? "Connecting..." : (googleAuthModel.authenticated ? "Disconnect" : "Connect")
                                enabled: !googleAuthModel.loading
                                Layout.preferredWidth: 110
                                Layout.preferredHeight: 36

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: {
                                        if (!parent.enabled) return Theme.surfaceAlt
                                        if (googleAuthModel.authenticated) return Theme.error + "20"
                                        return parent.hovered ? Theme.primaryHover : Theme.primary
                                    }
                                    border.color: googleAuthModel.authenticated ? Theme.error : "transparent"
                                    border.width: googleAuthModel.authenticated ? 1 : 0
                                }

                                contentItem: Label {
                                    text: parent.text
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: true
                                    color: {
                                        if (!parent.enabled) return Theme.textMuted
                                        if (googleAuthModel.authenticated) return Theme.error
                                        return Theme.primaryText
                                    }
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }

                                onClicked: {
                                    if (googleAuthModel.authenticated) {
                                        googleAuthModel.sign_out()
                                    } else {
                                        googleAuthModel.authenticate()
                                    }
                                }
                            }
                        }
                    }

                    // Google Error message
                    Label {
                        visible: googleAuthModel.error_message !== ""
                        text: googleAuthModel.error_message
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.error
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Info text
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingSm
                        height: 56
                        radius: Theme.inputRadius
                        color: Theme.surfaceAlt

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingSm
                            spacing: 2

                            RowLayout {
                                Text {
                                    text: Icons.info
                                    font.family: Icons.family
                                    font.pixelSize: Theme.fontSizeNormal
                                    color: Theme.textMuted
                                }

                                Label {
                                    text: "GitHub enables project tracking and repository management"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                            }

                            RowLayout {
                                Item { width: Theme.fontSizeNormal + 4 }
                                Label {
                                    text: "Google enables Gmail and Calendar integration"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                            }
                        }
                    }
                }
            }

            // About Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: aboutContent.implicitHeight + Theme.spacingMd * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: aboutContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    spacing: Theme.spacingMd

                    Label {
                        text: "About"
                        font.pixelSize: Theme.fontSizeMedium
                        font.bold: true
                        color: Theme.text
                    }

                    RowLayout {
                        spacing: Theme.spacingMd

                        Rectangle {
                            width: 64
                            height: 64
                            radius: 12
                            color: Theme.primary

                            Label {
                                anchors.centerIn: parent
                                text: "M"
                                font.pixelSize: 32
                                font.bold: true
                                color: Theme.primaryText
                            }
                        }

                        ColumnLayout {
                            spacing: 2

                            Label {
                                text: "MyMe"
                                font.pixelSize: Theme.fontSizeLarge
                                font.bold: true
                                color: Theme.text
                            }

                            Label {
                                text: "Personal Productivity & Dev Hub"
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textSecondary
                            }

                            Label {
                                text: "Version 0.1.0"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textMuted
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: Theme.border
                    }

                    Label {
                        text: "Built with Rust, Qt/QML, and cxx-qt"
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.textSecondary
                    }
                }
            }

            Item {
                Layout.fillHeight: true
            }
        }
    }
}
