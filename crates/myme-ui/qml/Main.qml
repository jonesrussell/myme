import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.myme 1.0

Kirigami.ApplicationWindow {
    id: root

    width: 1200
    height: 800
    title: "MyMe - Personal Productivity Hub"

    globalDrawer: Kirigami.GlobalDrawer {
        title: "MyMe"
        titleIcon: "applications-utilities"

        actions: [
            Kirigami.Action {
                text: "Notes"
                icon.name: "view-task"
                onTriggered: {
                    pageStack.clear()
                    pageStack.push(Qt.resolvedUrl("pages/NotePage.qml"))
                }
            },
            Kirigami.Action {
                text: "Repos"
                icon.name: "folder-git"
                enabled: true
                onTriggered: {
                    pageStack.clear()
                    pageStack.push(Qt.resolvedUrl("pages/RepoPage.qml"))
                }
            },
            Kirigami.Action {
                text: "Email"
                icon.name: "mail-message"
                enabled: false
                onTriggered: {
                    // Will be implemented in Phase 3
                }
            },
            Kirigami.Action {
                text: "Calendar"
                icon.name: "view-calendar"
                enabled: false
                onTriggered: {
                    // Will be implemented in Phase 3
                }
            },
            Kirigami.Action {
                text: "New Project"
                icon.name: "document-new"
                enabled: false
                onTriggered: {
                    // Will be implemented in Phase 4
                }
            }
        ]
    }

    pageStack.initialPage: Kirigami.Page {
        title: "Welcome"

        ColumnLayout {
            anchors.centerIn: parent
            spacing: Kirigami.Units.largeSpacing

            Kirigami.Icon {
                source: "applications-utilities"
                Layout.preferredWidth: Kirigami.Units.iconSizes.enormous
                Layout.preferredHeight: Kirigami.Units.iconSizes.enormous
                Layout.alignment: Qt.AlignHCenter
            }

            Kirigami.Heading {
                text: "MyMe"
                level: 1
                Layout.alignment: Qt.AlignHCenter
            }

            Controls.Label {
                text: "Your Personal Productivity & Dev Hub"
                Layout.alignment: Qt.AlignHCenter
            }

            Controls.Button {
                text: "View Notes"
                icon.name: "view-task"
                Layout.alignment: Qt.AlignHCenter
                onClicked: {
                    pageStack.clear()
                    pageStack.push(Qt.resolvedUrl("pages/NotePage.qml"))
                }
            }
        }
    }
}
