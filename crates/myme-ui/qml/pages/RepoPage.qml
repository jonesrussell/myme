import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.myme 1.0

Kirigami.ScrollablePage {
    id: repoPage
    title: "GitHub Repositories"

    RepoModel {
        id: repoModel
        Component.onCompleted: {
            // Check if authenticated and fetch repos
            if (authenticated) {
                fetchRepositories()
            }
        }
    }

    actions: [
        Kirigami.Action {
            text: repoModel && repoModel.authenticated ? "Sign Out" : "Sign In"
            icon.name: repoModel && repoModel.authenticated ? "system-log-out" : "system-log-in"
            onTriggered: {
                if (repoModel.authenticated) {
                    repoModel.signOut()
                } else {
                    repoModel.authenticate()
                }
            }
        },
        Kirigami.Action {
            text: "Refresh"
            icon.name: "view-refresh"
            enabled: repoModel && repoModel.authenticated && !repoModel.loading
            onTriggered: repoModel.fetchRepositories()
        },
        Kirigami.Action {
            text: "New Repository"
            icon.name: "list-add"
            enabled: repoModel && repoModel.authenticated && !repoModel.loading
            onTriggered: newRepoDialog.open()
        }
    ]

    ColumnLayout {
        width: parent.width
        spacing: Kirigami.Units.largeSpacing

        // Authentication status banner
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: repoModel && !repoModel.authenticated
            type: Kirigami.MessageType.Information
            text: "Sign in to GitHub to view and manage your repositories"
            actions: [
                Kirigami.Action {
                    text: "Sign In"
                    icon.name: "system-log-in"
                    onTriggered: repoModel.authenticate()
                }
            ]
        }

        // Error message
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: repoModel && repoModel.errorMessage !== ""
            type: Kirigami.MessageType.Error
            text: repoModel ? repoModel.errorMessage : ""
        }

        // Loading indicator
        Controls.BusyIndicator {
            Layout.alignment: Qt.AlignHCenter
            visible: repoModel && repoModel.loading
            running: repoModel && repoModel.loading
        }

        // User info
        RowLayout {
            Layout.fillWidth: true
            visible: repoModel && repoModel.authenticated && repoModel.username !== ""
            spacing: Kirigami.Units.smallSpacing

            Controls.Label {
                text: "Signed in as:"
                font.bold: true
            }

            Controls.Label {
                text: repoModel ? repoModel.username : ""
            }
        }

        // Repository list
        Repeater {
            model: repoModel ? repoModel.rowCount() : 0

            delegate: Kirigami.Card {
                Layout.fillWidth: true

                header: RowLayout {
                    Kirigami.Heading {
                        text: repoModel.getFullName(index)
                        level: 3
                        Layout.fillWidth: true
                    }

                    Controls.Label {
                        text: repoModel.getIsPrivate(index) ? "Private" : "Public"
                        color: repoModel.getIsPrivate(index) ?
                            Kirigami.Theme.neutralTextColor :
                            Kirigami.Theme.positiveTextColor
                    }
                }

                contentItem: ColumnLayout {
                    spacing: Kirigami.Units.smallSpacing

                    Controls.Label {
                        text: repoModel.getDescription(index)
                        visible: repoModel.getDescription(index) !== ""
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    RowLayout {
                        spacing: Kirigami.Units.largeSpacing

                        RowLayout {
                            spacing: Kirigami.Units.smallSpacing
                            Controls.Label {
                                text: "★"
                                color: Kirigami.Theme.highlightColor
                            }
                            Controls.Label {
                                text: repoModel.getStars(index)
                            }
                        }
                    }
                }

                actions: [
                    Kirigami.Action {
                        text: "Open in Browser"
                        icon.name: "internet-web-browser"
                        onTriggered: Qt.openUrlExternally(repoModel.getUrl(index))
                    },
                    Kirigami.Action {
                        text: "Copy Clone URL"
                        icon.name: "edit-copy"
                        onTriggered: {
                            // Copy to clipboard
                            clipboardHelper.text = repoModel.getCloneUrl(index)
                            clipboardHelper.selectAll()
                            clipboardHelper.copy()
                            showPassiveNotification("Clone URL copied to clipboard")
                        }
                    }
                ]
            }
        }

        // Placeholder when no repos
        Kirigami.PlaceholderMessage {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: repoModel && repoModel.authenticated &&
                     repoModel.rowCount() === 0 && !repoModel.loading
            text: "No repositories found"
            explanation: "Create a new repository to get started"
            icon.name: "folder-git"
        }
    }

    // Hidden text field for clipboard operations
    Controls.TextField {
        id: clipboardHelper
        visible: false
    }

    // New repository dialog
    Controls.Dialog {
        id: newRepoDialog
        title: "Create New Repository"
        modal: true
        standardButtons: Controls.Dialog.Ok | Controls.Dialog.Cancel
        anchors.centerIn: parent
        width: Math.min(500, parent.width - Kirigami.Units.gridUnit * 4)

        onAccepted: {
            if (repoNameField.text.trim() !== "") {
                repoModel.createRepository(
                    repoNameField.text.trim(),
                    repoDescField.text.trim(),
                    privateCheckBox.checked
                )
                repoNameField.text = ""
                repoDescField.text = ""
                privateCheckBox.checked = false
            }
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.smallSpacing

            Controls.Label {
                text: "Repository Name:"
            }

            Controls.TextField {
                id: repoNameField
                Layout.fillWidth: true
                placeholderText: "my-awesome-project"
            }

            Controls.Label {
                text: "Description (optional):"
                Layout.topMargin: Kirigami.Units.smallSpacing
            }

            Controls.TextField {
                id: repoDescField
                Layout.fillWidth: true
                placeholderText: "A short description of your project"
            }

            Controls.CheckBox {
                id: privateCheckBox
                text: "Private repository"
                checked: false
                Layout.topMargin: Kirigami.Units.smallSpacing
            }
        }
    }
}
