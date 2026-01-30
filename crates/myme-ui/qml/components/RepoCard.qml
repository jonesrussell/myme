import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: card
    required property int index
    required property var repoModel

    implicitHeight: cardContent.implicitHeight + Theme.spacingMd * 2
    radius: Theme.cardRadius
    color: cardMouseArea.containsMouse ? Theme.surfaceHover : Theme.surface
    border.color: cardMouseArea.containsMouse ? Theme.primary : Theme.border
    border.width: 1

    Behavior on color { ColorAnimation { duration: 100 } }
    Behavior on border.color { ColorAnimation { duration: 100 } }

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
