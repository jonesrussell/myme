import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: card
    required property int index
    required property var repoModel
    property var projectModel: null

    implicitHeight: cardContent.implicitHeight + Theme.spacingMd * 2
    radius: Theme.cardRadius
    color: cardMouseArea.containsMouse ? Theme.surfaceHover : Theme.surface
    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
    border.width: 1

    scale: cardMouseArea.containsMouse ? 1.01 : 1.0
    Behavior on scale { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
    Behavior on color { ColorAnimation { duration: 100 } }

    opacity: 0
    Component.onCompleted: cardEntryAnim.start()
    SequentialAnimation {
        id: cardEntryAnim
        PauseAnimation { duration: card.index * 30 }
        ParallelAnimation {
            NumberAnimation { target: card; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
            NumberAnimation { target: card; property: "y"; from: card.y + 8; to: card.y; duration: 200; easing.type: Easing.OutCubic }
        }
    }

    MouseArea {
        id: cardMouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
    }

    ColumnLayout {
        id: cardContent
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingSm

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingMd

            ColumnLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingXs

                Label {
                    text: repoModel ? repoModel.getFullName(index) : ""
                    font.pixelSize: Theme.fontSizeMedium
                    font.bold: true
                    color: Theme.text
                    Layout.fillWidth: true
                }

                Label {
                    text: {
                        const path = repoModel ? repoModel.getLocalPath(index) : "";
                        const branch = repoModel ? repoModel.getBranch(index) : "";
                        if (path) return path;
                        if (branch) return branch;
                        return "";
                    }
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                    Layout.fillWidth: true
                    visible: text !== ""
                }
            }

            Label {
                text: {
                    const s = repoModel ? repoModel.getState(index) : 0;
                    if (s === 0) return "Local only";
                    if (s === 1) return "GitHub";
                    return "Local + GitHub";
                }
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingMd

            Label {
                text: repoModel && repoModel.getIsClean(index) ? "Clean" : "Dirty"
                font.pixelSize: Theme.fontSizeSmall
                color: repoModel && repoModel.getIsClean(index) ? Theme.success : Theme.warning
            }

            Item { Layout.fillWidth: true }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            Button {
                visible: repoModel && repoModel.getHasGithub(index) && !repoModel.getHasLocal(index)
                enabled: repoModel && !repoModel.getBusy(index)
                text: "Clone"
                onClicked: repoModel.cloneRepo(index)
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

            Button {
                visible: repoModel && repoModel.getHasLocal(index)
                enabled: repoModel && !repoModel.getBusy(index)
                text: "Pull"
                onClicked: repoModel.pullRepo(index)
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

            Button {
                visible: repoModel && repoModel.getHasGithub(index)
                text: "Open"
                onClicked: {
                    const url = repoModel.getHtmlUrl(index);
                    if (url) Qt.openUrlExternally(url);
                }
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

            Button {
                visible: repoModel && repoModel.getHasGithub(index) && projectModel && projectModel.row_count() > 0
                text: "Add to project"
                onClicked: addToProjectMenu.open()
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

            Menu {
                id: addToProjectMenu
                y: parent.height
                width: Math.min(250, card.width)

                Repeater {
                    model: projectModel ? projectModel.row_count() : 0
                    delegate: MenuItem {
                        text: projectModel ? projectModel.get_project_name(modelData) : ""
                        onTriggered: {
                            const repoId = repoModel.getFullName(index);
                            const projectId = projectModel.get_id(modelData);
                            if (repoId && projectId) {
                                projectModel.add_repo_to_project_by_id(projectId, repoId);
                            }
                        }
                    }
                }
            }

            Button {
                visible: repoModel && repoModel.getHasGithub(index)
                text: "Copy URL"
                onClicked: {
                    const url = repoModel.getCloneUrl(index);
                    if (url) {
                        clipboardHelper.text = url;
                        clipboardHelper.selectAll();
                        clipboardHelper.copy();
                    }
                }
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

            Item { Layout.fillWidth: true }

            Button {
                visible: repoModel && repoModel.getBusy(index)
                text: "Cancel"
                onClicked: repoModel.cancel_operation()
                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.errorHover : Theme.error
                }
                contentItem: Label {
                    text: parent.text
                    color: Theme.primaryText
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }

            BusyIndicator {
                visible: repoModel && repoModel.getBusy(index)
                running: visible
                Layout.alignment: Qt.AlignRight
            }
        }
    }

    TextField {
        id: clipboardHelper
        visible: false
    }
}
