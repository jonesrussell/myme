import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Page {
    id: settingsPage
    title: "Settings"

    background: Rectangle {
        color: Theme.background
    }

    header: ToolBar {
        background: Rectangle {
            color: Theme.surface
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
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
                Layout.preferredHeight: appearanceContent.implicitHeight + Theme.spacingLg * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: appearanceContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLg
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
                                        text: "☀️"
                                        font.pixelSize: 20
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
                                        text: "🌙"
                                        font.pixelSize: 20
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
                                        GradientStop { position: 0.0; color: "#f8f9fa" }
                                        GradientStop { position: 1.0; color: "#1a1a2e" }
                                    }
                                    border.color: "#888"
                                    border.width: 1

                                    Text {
                                        anchors.centerIn: parent
                                        text: "🔄"
                                        font.pixelSize: 20
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

                        Item { Layout.fillWidth: true }
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
                                text: Theme.mode === "auto"
                                    ? "Currently using " + (Theme.isDark ? "dark" : "light") + " theme (synced with system)"
                                    : "Using " + Theme.mode + " theme"
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                            }
                        }
                    }
                }
            }

            // About Section
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: aboutContent.implicitHeight + Theme.spacingLg * 2
                color: Theme.surface
                border.color: Theme.border
                border.width: 1
                radius: Theme.cardRadius

                ColumnLayout {
                    id: aboutContent
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLg
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

            Item { Layout.fillHeight: true }
        }
    }
}
