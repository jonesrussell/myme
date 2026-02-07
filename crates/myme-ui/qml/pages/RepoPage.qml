import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."
import "../components"

Page {
    id: repoPage
    title: "Repos"

    property int repoCount: 0

    background: Rectangle {
        color: Theme.background
    }

    RepoModel {
        id: repoModel
    }

    ProjectModel {
        id: projectModel
    }

    AuthModel {
        id: authModel
    }

    Timer {
        id: projectPollTimer
        interval: 100
        running: projectModel.loading
        repeat: true
        onTriggered: projectModel.poll_channel()
    }

    Timer {
        interval: 100
        running: true
        repeat: true
        onTriggered: repoModel.poll_channel()
    }

    Connections {
        target: repoModel
        function onLoadingChanged() {
            if (!repoModel.loading) {
                repoPage.repoCount = repoModel.rowCount();
            }
        }
        function onReposChanged() {
            repoPage.repoCount = repoModel.rowCount();
        }
        function onAuthenticatedChanged() {
            repoModel.fetchRepos();
        }
    }

    Connections {
        target: authModel
        function onAuth_completed() {
            repoModel.checkAuth();
            repoModel.fetchRepos();
        }
    }

    Component.onCompleted: {
        repoModel.checkAuth();
        projectModel.check_auth();
        projectModel.fetch_projects();
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Repos"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingMd
            }

            ToolButton {
                text: Icons.arrowsClockwise
                font.family: Icons.family
                font.pixelSize: 18
                enabled: !repoModel.loading
                onClicked: repoModel.fetchRepos()
                ToolTip.text: "Refresh"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                    opacity: parent.enabled ? 1.0 : 0.5
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: Theme.text
                    font.pixelSize: 18
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    opacity: parent.enabled ? 1.0 : 0.5
                }
            }

            Item { width: Theme.spacingSm }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        spacing: Theme.spacingMd

        Rectangle {
            visible: !repoModel.authenticated
            Layout.fillWidth: true
            Layout.preferredHeight: 56
            color: Theme.infoBg
            border.color: Theme.info
            border.width: 1
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingMd

                Label {
                    text: "Sign in to GitHub to see your remote repositories."
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.text
                    Layout.fillWidth: true
                }

                Button {
                    text: "Sign in"
                    onClicked: authModel.authenticate()
                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.primaryHover : Theme.primary
                    }
                    contentItem: Label {
                        text: parent.text
                        color: Theme.primaryText
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        Rectangle {
            visible: repoModel.configPathInvalid
            Layout.fillWidth: true
            Layout.preferredHeight: 56
            color: Theme.warningBg
            border.color: Theme.warning
            border.width: 1
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingMd

                Label {
                    text: "Repository path is invalid (missing or not a directory). Using: " + (repoModel.effectivePath || ".")
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.text
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
            }
        }

        Rectangle {
            visible: repoModel.errorMessage !== ""
            Layout.fillWidth: true
            Layout.preferredHeight: 56
            color: Theme.errorBg
            border.color: Theme.error
            border.width: 1
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingMd

                Label {
                    text: repoModel.errorMessage
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.error
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }

                Button {
                    text: "Dismiss"
                    onClicked: repoModel.clearError()
                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                    }
                    contentItem: Label {
                        text: parent.text
                        color: Theme.text
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        BusyIndicator {
            visible: repoModel.loading
            running: repoModel.loading
            Layout.alignment: Qt.AlignHCenter
        }

        ScrollView {
            visible: !repoModel.loading && repoCount > 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ColumnLayout {
                width: parent.width - 20
                spacing: Theme.spacingMd

                Repeater {
                    model: repoCount
                    delegate: RepoCard {
                        index: modelData
                        repoModel: repoModel
                        projectModel: projectModel
                        Layout.fillWidth: true
                    }
                }
            }
        }

        Rectangle {
            visible: !repoModel.loading && repoCount === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.surface
            border.color: Theme.border
            border.width: 1
            radius: Theme.cardRadius

            ColumnLayout {
                anchors.centerIn: parent
                spacing: Theme.spacingMd

                Label {
                    text: Icons.folderSimple
                    font.family: Icons.family
                    font.pixelSize: 64
                    color: Theme.textMuted
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "No repositories found"
                    font.pixelSize: Theme.fontSizeLarge
                    font.bold: true
                    color: Theme.text
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: repoModel.authenticated
                        ? "Add local repos under your dev path, or clone from GitHub."
                        : "Sign in to GitHub to see remote repos. Local repos from your dev path will still appear."
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignHCenter
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
    }
}
