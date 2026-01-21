import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 1200
    height: 800
    visible: true
    title: "MyMe - Personal Productivity Hub"

    property bool sidebarCollapsed: false
    property int sidebarExpandedWidth: 200
    property int sidebarCollapsedWidth: 50

    header: ToolBar {
        RowLayout {
            anchors.fill: parent

            ToolButton {
                text: sidebarCollapsed ? "▶" : "◀"
                onClicked: sidebarCollapsed = !sidebarCollapsed
            }

            Label {
                text: "MyMe"
                elide: Label.ElideRight
                horizontalAlignment: Qt.AlignHCenter
                verticalAlignment: Qt.AlignVCenter
                Layout.fillWidth: true
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Always-visible sidebar
        Rectangle {
            id: sidebar
            Layout.fillHeight: true
            Layout.preferredWidth: sidebarCollapsed ? sidebarCollapsedWidth : sidebarExpandedWidth
            color: palette.base

            Behavior on Layout.preferredWidth {
                NumberAnimation { duration: 150; easing.type: Easing.OutQuad }
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 5
                spacing: 5

                Label {
                    text: sidebarCollapsed ? "M" : "MyMe"
                    font.pixelSize: sidebarCollapsed ? 18 : 24
                    font.bold: true
                    Layout.fillWidth: true
                    horizontalAlignment: sidebarCollapsed ? Text.AlignHCenter : Text.AlignLeft
                }

                Button {
                    text: sidebarCollapsed ? "📝" : "Notes"
                    Layout.fillWidth: true
                    flat: true
                    onClicked: stackView.replace("pages/NotePage.qml")

                    ToolTip.visible: sidebarCollapsed && hovered
                    ToolTip.text: "Notes"
                }

                Button {
                    text: sidebarCollapsed ? "📁" : "Repos"
                    Layout.fillWidth: true
                    flat: true
                    enabled: false
                    onClicked: stackView.replace("pages/RepoPage.qml")

                    ToolTip.visible: sidebarCollapsed && hovered
                    ToolTip.text: "Repos"
                }

                Button {
                    text: sidebarCollapsed ? "🔧" : "Dev Tools"
                    Layout.fillWidth: true
                    flat: true
                    onClicked: stackView.replace("pages/DevToolsPage.qml")

                    ToolTip.visible: sidebarCollapsed && hovered
                    ToolTip.text: "Dev Tools"
                }

                Item { Layout.fillHeight: true }
            }
        }

        // Separator line
        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            color: palette.mid
        }

        // Main content area
        StackView {
            id: stackView
            Layout.fillWidth: true
            Layout.fillHeight: true

            initialItem: Page {
                title: "Welcome"

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 20

                    Label {
                        text: "MyMe"
                        font.pixelSize: 48
                        font.bold: true
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Label {
                        text: "Your Personal Productivity & Dev Hub"
                        font.pixelSize: 16
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Button {
                        text: "View Notes"
                        Layout.alignment: Qt.AlignHCenter
                        onClicked: stackView.replace("pages/NotePage.qml")
                    }
                }
            }
        }
    }
}
