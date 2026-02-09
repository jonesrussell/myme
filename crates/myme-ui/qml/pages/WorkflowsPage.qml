import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: workflowsPage
    title: "CI/CD Workflows"

    background: Rectangle {
        color: Theme.background
    }

    WorkflowModel {
        id: workflowModel
    }

    Timer {
        id: workflowPollTimer
        interval: 100
        running: workflowModel.loading
        repeat: true
        onTriggered: workflowModel.poll_channel()
    }

    Component.onCompleted: {
        workflowModel.check_auth();
        if (workflowModel.authenticated) {
            workflowModel.fetch_workflows();
        }
    }

    Connections {
        target: workflowModel
        function onAuthenticatedChanged() {
            if (workflowModel.authenticated) {
                workflowModel.fetch_workflows();
            }
        }
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            ToolButton {
                text: Icons.caretLeft
                font.family: Icons.family
                font.pixelSize: 18
                onClicked: AppContext.pageStack.pop()
                ToolTip.text: "Back to Projects"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: Theme.text
                    font.pixelSize: 18
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }

            Label {
                text: "CI/CD Workflows"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                leftPadding: Theme.spacingSm
            }

            ToolButton {
                text: Icons.arrowsClockwise
                font.family: Icons.family
                font.pixelSize: 18
                enabled: workflowModel.authenticated && !workflowModel.loading
                onClicked: workflowModel.fetch_workflows()
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

        // Not authenticated
        Rectangle {
            visible: !workflowModel.authenticated
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.surface
            border.color: Theme.isDark ? "#ffffff08" : "#00000008"
            border.width: 1
            radius: Theme.cardRadius

            ColumnLayout {
                anchors.centerIn: parent
                spacing: Theme.spacingMd

                Label {
                    text: Icons.githubLogo
                    font.family: Icons.family
                    font.pixelSize: 64
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Connect GitHub to view workflows"
                    font.pixelSize: Theme.fontSizeMedium
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Link repos to projects, then open this page to see their CI/CD workflows."
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.textMuted
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                    Layout.preferredWidth: 320
                    Layout.alignment: Qt.AlignHCenter
                }
            }
        }

        // Error message
        Rectangle {
            visible: workflowModel.authenticated && workflowModel.error_message.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: 60
            color: Theme.errorBg
            border.color: "transparent"
            border.width: 0
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingSm

                Label {
                    text: workflowModel.error_message
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.error
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }

                Button {
                    text: "Retry"
                    onClicked: workflowModel.fetch_workflows()
                }
            }
        }

        // Loading (no data yet)
        BusyIndicator {
            visible: workflowModel.authenticated && workflowModel.loading && workflowModel.row_count() === 0
            running: workflowModel.loading
            Layout.alignment: Qt.AlignCenter
        }

        // Authenticated but no linked repos
        Rectangle {
            visible: workflowModel.authenticated && !workflowModel.loading && workflowModel.row_count() === 0 && workflowModel.error_message.length === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.surface
            border.color: Theme.isDark ? "#ffffff08" : "#00000008"
            border.width: 1
            radius: Theme.cardRadius

            ColumnLayout {
                anchors.centerIn: parent
                spacing: Theme.spacingMd

                Label {
                    text: "No project repos yet"
                    font.pixelSize: Theme.fontSizeMedium
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Add GitHub repos to your projects to see their CI/CD workflows here."
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.textMuted
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                    Layout.preferredWidth: 280
                    Layout.alignment: Qt.AlignHCenter
                }
            }
        }

        // List of repos and their workflows
        ScrollView {
            visible: workflowModel.authenticated && workflowModel.row_count() > 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ColumnLayout {
                width: parent.width - (parent.ScrollBar.vertical?.width ?? 0)
                spacing: Theme.spacingLg

                Repeater {
                    model: workflowModel.row_count()

                    Rectangle {
                        id: workflowDelegate
                        required property int index
                        property int repoIndex: index
                        Layout.fillWidth: true
                        Layout.preferredHeight: repoColumn.implicitHeight + Theme.spacingMd * 2
                        color: Theme.surface
                        border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                        border.width: 1
                        radius: Theme.cardRadius

                        opacity: 0
                        Component.onCompleted: wfEntryAnim.start()
                        SequentialAnimation {
                            id: wfEntryAnim
                            PauseAnimation { duration: workflowDelegate.index * 30 }
                            ParallelAnimation {
                                NumberAnimation { target: workflowDelegate; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                                NumberAnimation { target: workflowDelegate; property: "y"; from: workflowDelegate.y + 8; to: workflowDelegate.y; duration: 200; easing.type: Easing.OutCubic }
                            }
                        }

                        ColumnLayout {
                            id: repoColumn
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMd
                            spacing: Theme.spacingSm

                            Label {
                                text: workflowModel.get_repo_id(repoColumn.parent.repoIndex)
                                font.pixelSize: Theme.fontSizeMedium
                                font.bold: true
                                color: Theme.text
                                Layout.fillWidth: true
                            }

                            Repeater {
                                model: workflowModel.get_workflow_count(repoColumn.parent.repoIndex)

                                Rectangle {
                                    property int workflowIndex: index
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: workflowRow.implicitHeight + Theme.spacingXs * 2
                                    color: Theme.surfaceAlt
                                    radius: Theme.buttonRadius

                                    RowLayout {
                                        id: workflowRow
                                        anchors.fill: parent
                                        anchors.margins: Theme.spacingXs
                                        spacing: Theme.spacingSm

                                        Label {
                                            text: workflowModel.get_workflow_name(repoColumn.parent.repoIndex, workflowRow.parent.workflowIndex)
                                            font.pixelSize: Theme.fontSizeNormal
                                            color: Theme.text
                                            Layout.preferredWidth: 140
                                            elide: Text.ElideRight
                                        }

                                        Label {
                                            text: workflowModel.get_workflow_path(repoColumn.parent.repoIndex, workflowRow.parent.workflowIndex)
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.textSecondary
                                            Layout.fillWidth: true
                                            elide: Text.ElideMiddle
                                        }

                                        Label {
                                            text: workflowModel.get_workflow_state(repoColumn.parent.repoIndex, workflowRow.parent.workflowIndex)
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.textMuted
                                            Layout.alignment: Qt.AlignRight
                                        }

                                        ToolButton {
                                            text: Icons.cornersOut
                                            font.family: Icons.family
                                            font.pixelSize: 16
                                            visible: workflowModel.get_workflow_html_url(repoColumn.parent.repoIndex, workflowRow.parent.workflowIndex).length > 0
                                            onClicked: {
                                                const url = workflowModel.get_workflow_html_url(repoColumn.parent.repoIndex, workflowRow.parent.workflowIndex);
                                                if (url.length > 0) {
                                                    Qt.openUrlExternally(url);
                                                }
                                            }
                                            ToolTip.text: "Open on GitHub"
                                            ToolTip.visible: hovered

                                            contentItem: Text {
                                                text: parent.text
                                                font.family: Icons.family
                                                color: Theme.text
                                                font.pixelSize: 16
                                                horizontalAlignment: Text.AlignHCenter
                                                verticalAlignment: Text.AlignVCenter
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
