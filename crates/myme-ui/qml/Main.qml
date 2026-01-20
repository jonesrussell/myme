import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 1200
    height: 800
    visible: true
    title: "MyMe - Personal Productivity Hub"

    // Simple drawer menu
    Drawer {
        id: drawer
        width: 250
        height: root.height

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 10
            spacing: 5

            Label {
                text: "MyMe"
                font.pixelSize: 24
                font.bold: true
                Layout.fillWidth: true
            }

            Button {
                text: "Notes"
                Layout.fillWidth: true
                onClicked: {
                    stackView.replace("pages/NotePage.qml")
                    drawer.close()
                }
            }

            Button {
                text: "Repos"
                Layout.fillWidth: true
                enabled: false
                onClicked: {
                    stackView.replace("pages/RepoPage.qml")
                    drawer.close()
                }
            }

            Item { Layout.fillHeight: true }
        }
    }

    header: ToolBar {
        RowLayout {
            anchors.fill: parent

            ToolButton {
                text: "☰"
                onClicked: drawer.open()
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

    StackView {
        id: stackView
        anchors.fill: parent

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
