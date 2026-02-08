import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: projectsPage
    title: "Projects"

    // Track project count for GridView model
    property int projectCount: 0

    background: Rectangle {
        color: Theme.background
    }

    // Instantiate the ProjectModel from Rust
    ProjectModel {
        id: projectModel
    }

    // Auth model for handling OAuth flow
    AuthModel {
        id: authModel
    }

    // Timer to poll for async project operation results
    Timer {
        id: projectPollTimer
        interval: 100
        running: projectModel.loading
        repeat: true
        onTriggered: projectModel.poll_channel()
    }

    // Timer to poll for async auth operation results
    Timer {
        id: authPollTimer
        interval: 100
        running: authModel.loading
        repeat: true
        onTriggered: authModel.poll_channel()
    }

    // Update project count when loading finishes
    Connections {
        target: projectModel
        function onLoadingChanged() {
            if (!projectModel.loading) {
                projectsPage.projectCount = projectModel.row_count();
            }
        }
        function onProjects_changed() {
            projectsPage.projectCount = projectModel.row_count();
        }
        function onAuthenticatedChanged() {
            // Re-fetch projects when auth status changes
            if (projectModel.authenticated) {
                projectModel.fetch_projects();
            }
        }
    }

    // Listen for auth completion to fetch projects
    Connections {
        target: authModel
        function onAuth_completed() {
            // After successful auth, check project auth and fetch
            projectModel.check_auth();
            if (projectModel.authenticated) {
                projectModel.fetch_projects();
            }
        }
    }

    // Status bar colors for task distribution
    readonly property var statusColors: ({
        backlog: "#9e9e9e",
        todo: "#2196f3",
        inprogress: "#ff9800",
        blocked: "#f44336",
        review: "#9c27b0",
        done: "#4caf50"
    })

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Projects"
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
                enabled: projectModel.authenticated && !projectModel.loading
                onClicked: {
                    // Sync all projects
                    for (let i = 0; i < projectModel.row_count(); i++) {
                        projectModel.sync_project(i);
                    }
                }
                ToolTip.text: "Sync All"
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

            ToolButton {
                text: Icons.plus
                font.family: Icons.family
                font.pixelSize: 18
                enabled: projectModel.authenticated
                onClicked: addDialog.open()
                ToolTip.text: "Create Project"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.primary : Theme.surfaceHover
                    opacity: parent.enabled ? 1.0 : 0.5
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: parent.parent.hovered ? Theme.primaryText : Theme.text
                    font.pixelSize: 18
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    opacity: parent.enabled ? 1.0 : 0.5
                }
            }

            Item { width: Theme.spacingSm }
        }
    }

    // Main content
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        spacing: Theme.spacingMd

        // Not authenticated state
        Rectangle {
            visible: !projectModel.authenticated
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
                    text: Icons.githubLogo
                    font.family: Icons.family
                    font.pixelSize: 64
                    color: Theme.textMuted
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "GitHub Authentication Required"
                    font.pixelSize: Theme.fontSizeLarge
                    font.bold: true
                    color: Theme.text
                    Layout.alignment: Qt.AlignHCenter
                }

                Label {
                    text: "Connect your GitHub account to manage project tasks"
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.textSecondary
                    Layout.alignment: Qt.AlignHCenter
                }

                Rectangle {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: Theme.spacingMd
                    width: authLabel.implicitWidth + Theme.spacingLg * 2
                    height: authLabel.implicitHeight + Theme.spacingMd
                    radius: Theme.buttonRadius
                    color: authModel.loading ? Theme.surfaceAlt : (authMouseArea.containsMouse ? Theme.primaryHover : Theme.primary)
                    opacity: authModel.loading ? 0.7 : 1.0

                    RowLayout {
                        anchors.centerIn: parent
                        spacing: Theme.spacingSm

                        Label {
                            text: authModel.loading ? Icons.spinner : Icons.githubLogo
                            font.family: Icons.family
                            font.pixelSize: 18
                            color: Theme.primaryText

                            RotationAnimation on rotation {
                                running: authModel.loading
                                from: 0
                                to: 360
                                duration: 1000
                                loops: Animation.Infinite
                            }
                        }

                        Label {
                            id: authLabel
                            text: authModel.loading ? "Connecting..." : "Connect GitHub"
                            font.pixelSize: Theme.fontSizeNormal
                            font.bold: true
                            color: Theme.primaryText
                        }
                    }

                    MouseArea {
                        id: authMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: authModel.loading ? Qt.BusyCursor : Qt.PointingHandCursor
                        enabled: !authModel.loading
                        onClicked: authModel.authenticate()
                    }
                }

                // Error message from auth
                Label {
                    visible: authModel.error_message !== ""
                    text: authModel.error_message
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.error
                    wrapMode: Text.WordWrap
                    Layout.alignment: Qt.AlignHCenter
                    Layout.maximumWidth: 300
                }
            }
        }

        // Error message banner
        Rectangle {
            visible: projectModel.authenticated && projectModel.error_message.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: 60
            color: Theme.errorBg
            border.color: Theme.error
            border.width: 1
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingMd

                Label {
                    text: Icons.warning
                    font.family: Icons.family
                    font.pixelSize: 20
                    color: Theme.error
                }

                Label {
                    text: projectModel.error_message
                    color: Theme.error
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                }

                Button {
                    text: "Retry"
                    onClicked: projectModel.fetch_projects()

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.primaryHover : Theme.primary
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.primaryText
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        // Loading indicator
        BusyIndicator {
            visible: projectModel.authenticated && projectModel.loading && projectsPage.projectCount === 0
            running: projectModel.loading
            Layout.alignment: Qt.AlignHCenter
        }

        // Projects grid
        ScrollView {
            visible: projectModel.authenticated
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            GridLayout {
                id: projectsGrid
                width: parent.width
                columns: Math.max(1, Math.floor(width / 350))
                rowSpacing: Theme.spacingMd
                columnSpacing: Theme.spacingMd

                Repeater {
                    model: projectsPage.projectCount

                    delegate: Rectangle {
                        id: projectCard
                        required property int index

                        Layout.fillWidth: true
                        Layout.preferredHeight: cardContent.implicitHeight + Theme.spacingMd * 2
                        Layout.minimumWidth: 300
                        Layout.maximumWidth: 450
                        color: cardMouseArea.containsMouse ? Theme.surfaceHover : Theme.surface
                        border.color: cardMouseArea.containsMouse ? Theme.primary : Theme.border
                        border.width: 1
                        radius: Theme.cardRadius

                        Behavior on color {
                            ColorAnimation { duration: 100 }
                        }
                        Behavior on border.color {
                            ColorAnimation { duration: 100 }
                        }

                        MouseArea {
                            id: cardMouseArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                const projectId = projectModel.get_id(projectCard.index);
                                const projectName = projectModel.get_project_name(projectCard.index);
                                AppContext.pageStack.push(AppContext.pageUrl("ProjectDetailPage"), {
                                    projectId: projectId,
                                    projectName: projectName
                                });
                            }
                        }

                        ColumnLayout {
                            id: cardContent
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.margins: Theme.spacingMd
                            spacing: Theme.spacingSm

                            // Repository name and actions
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm

                                Label {
                                    text: Icons.githubLogo
                                    font.family: Icons.family
                                    font.pixelSize: 18
                                    color: Theme.textSecondary
                                }

                                Label {
                                    text: projectModel.get_project_name(projectCard.index)
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.bold: true
                                    color: Theme.text
                                    Layout.fillWidth: true
                                    elide: Text.ElideMiddle
                                }

                                // Refresh button
                                Rectangle {
                                    width: 28
                                    height: 28
                                    radius: Theme.buttonRadius
                                    color: refreshMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                                    Label {
                                        anchors.centerIn: parent
                                        text: Icons.arrowsClockwise
                                        font.family: Icons.family
                                        font.pixelSize: 14
                                        color: Theme.textSecondary
                                    }

                                    MouseArea {
                                        id: refreshMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: projectModel.sync_project(projectCard.index)
                                    }

                                    ToolTip.visible: refreshMouseArea.containsMouse
                                    ToolTip.text: "Sync with GitHub"
                                    ToolTip.delay: 500
                                }

                                // Delete button
                                Rectangle {
                                    width: 28
                                    height: 28
                                    radius: Theme.buttonRadius
                                    color: deleteMouseArea.containsMouse ? Theme.error + "30" : "transparent"

                                    Label {
                                        anchors.centerIn: parent
                                        text: Icons.trash
                                        font.family: Icons.family
                                        font.pixelSize: 14
                                        color: deleteMouseArea.containsMouse ? Theme.error : Theme.textSecondary
                                    }

                                    MouseArea {
                                        id: deleteMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            removeDialog.projectIndex = projectCard.index;
                                            removeDialog.projectName = projectModel.get_project_name(projectCard.index);
                                            removeDialog.open();
                                        }
                                    }

                                    ToolTip.visible: deleteMouseArea.containsMouse
                                    ToolTip.text: "Remove project"
                                    ToolTip.delay: 500
                                }
                            }

                            // Repos (if any)
                            Label {
                                visible: {
                                    const repos = projectModel.get_repos_for_project(projectCard.index);
                                    try {
                                        const arr = JSON.parse(repos);
                                        return arr && arr.length > 0;
                                    } catch (e) { return false; }
                                }
                                text: {
                                    const repos = projectModel.get_repos_for_project(projectCard.index);
                                    try {
                                        const arr = JSON.parse(repos);
                                        return (arr || []).join(", ");
                                    } catch (e) { return ""; }
                                }
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textSecondary
                                Layout.fillWidth: true
                                elide: Text.ElideMiddle
                            }

                            // Description
                            Label {
                                text: projectModel.get_description(projectCard.index) || "No description"
                                font.pixelSize: Theme.fontSizeNormal
                                color: Theme.textSecondary
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                            }

                            // Status bar showing task distribution
                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 8
                                radius: 4
                                color: Theme.surfaceAlt
                                clip: true

                                RowLayout {
                                    anchors.fill: parent
                                    spacing: 0

                                    property var counts: {
                                        const json = projectModel.get_task_counts(projectCard.index);
                                        try {
                                            return JSON.parse(json);
                                        } catch (e) {
                                            return { backlog: 0, todo: 0, in_progress: 0, blocked: 0, review: 0, done: 0 };
                                        }
                                    }

                                    property int total: counts.backlog + counts.todo + counts.in_progress + counts.blocked + counts.review + counts.done

                                    // Backlog segment
                                    Rectangle {
                                        visible: parent.counts.backlog > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.backlog / parent.total) * parent.width : 0
                                        color: statusColors.backlog
                                        radius: parent.children[0] === this ? 4 : 0
                                    }

                                    // Todo segment
                                    Rectangle {
                                        visible: parent.counts.todo > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.todo / parent.total) * parent.width : 0
                                        color: statusColors.todo
                                    }

                                    // In Progress segment
                                    Rectangle {
                                        visible: parent.counts.in_progress > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.in_progress / parent.total) * parent.width : 0
                                        color: statusColors.inprogress
                                    }

                                    // Blocked segment
                                    Rectangle {
                                        visible: parent.counts.blocked > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.blocked / parent.total) * parent.width : 0
                                        color: statusColors.blocked
                                    }

                                    // Review segment
                                    Rectangle {
                                        visible: parent.counts.review > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.review / parent.total) * parent.width : 0
                                        color: statusColors.review
                                    }

                                    // Done segment
                                    Rectangle {
                                        visible: parent.counts.done > 0 && parent.total > 0
                                        Layout.fillHeight: true
                                        Layout.preferredWidth: parent.total > 0 ? (parent.counts.done / parent.total) * parent.width : 0
                                        color: statusColors.done
                                        radius: 4
                                    }
                                }
                            }

                            // Task count and percentage done
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm

                                property var counts: {
                                    const json = projectModel.get_task_counts(projectCard.index);
                                    try {
                                        return JSON.parse(json);
                                    } catch (e) {
                                        return { backlog: 0, todo: 0, in_progress: 0, blocked: 0, review: 0, done: 0 };
                                    }
                                }

                                property int total: counts.backlog + counts.todo + counts.in_progress + counts.blocked + counts.review + counts.done
                                property int percentDone: total > 0 ? Math.round((counts.done / total) * 100) : 0

                                Label {
                                    text: parent.total + " tasks"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textMuted
                                }

                                Item { Layout.fillWidth: true }

                                Rectangle {
                                    visible: parent.total > 0
                                    width: doneLabel.implicitWidth + Theme.spacingSm * 2
                                    height: doneLabel.implicitHeight + Theme.spacingXs
                                    radius: 4
                                    color: statusColors.done + "30"

                                    Label {
                                        id: doneLabel
                                        anchors.centerIn: parent
                                        text: parent.parent.percentDone + "% done"
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.bold: true
                                        color: statusColors.done
                                    }
                                }
                            }
                        }
                    }
                }

                // Empty state
                Item {
                    visible: projectModel.authenticated && !projectModel.loading && projectsPage.projectCount === 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: 300
                    Layout.columnSpan: parent.columns

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: Theme.spacingMd

                        Label {
                            text: Icons.folder
                            font.family: Icons.family
                            font.pixelSize: 48
                            color: Theme.textMuted
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Label {
                            text: "No projects yet"
                            font.pixelSize: Theme.fontSizeLarge
                            font.bold: true
                            color: Theme.text
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Label {
                            text: "Click + to create your first project"
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textSecondary
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
            }
        }
    }

    // Create project dialog
    Dialog {
        id: addDialog
        title: "Create Project"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 450)
        height: 220

        background: Rectangle {
            color: Theme.surface
            border.color: Theme.border
            border.width: 1
            radius: Theme.cardRadius
        }

        header: Rectangle {
            color: Theme.surfaceAlt
            height: 50
            radius: Theme.cardRadius

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: Theme.cardRadius
                color: Theme.surfaceAlt
            }

            Label {
                anchors.centerIn: parent
                text: "Create Project"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
        }

        onAccepted: {
            if (nameField.text.trim().length > 0) {
                projectModel.create_project(nameField.text.trim(), descField.text.trim());
                nameField.text = "";
                descField.text = "";
            }
        }

        onRejected: {
            nameField.text = "";
            descField.text = "";
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Project Name:"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: Theme.inputBg
                border.color: nameField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: nameField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                Behavior on border.color {
                    ColorAnimation { duration: 100 }
                }

                TextField {
                    id: nameField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "My Project"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }

            Label {
                text: "Description (optional):"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: Theme.inputBg
                border.color: descField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: descField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                Behavior on border.color {
                    ColorAnimation { duration: 100 }
                }

                TextField {
                    id: descField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "Project description"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }

            Label {
                text: "Add GitHub repos in the project detail view"
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: addDialog.accept()
        }
    }

    // Remove confirmation dialog
    Dialog {
        id: removeDialog
        title: "Remove Project"
        standardButtons: Dialog.Yes | Dialog.No
        modal: true

        property int projectIndex: -1
        property string projectName: ""

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 400)
        height: 180

        background: Rectangle {
            color: Theme.surface
            border.color: Theme.border
            border.width: 1
            radius: Theme.cardRadius
        }

        header: Rectangle {
            color: Theme.errorBg
            height: 50
            radius: Theme.cardRadius

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: Theme.cardRadius
                color: Theme.errorBg
            }

            RowLayout {
                anchors.centerIn: parent
                spacing: Theme.spacingSm

                Label {
                    text: Icons.warning
                    font.family: Icons.family
                    font.pixelSize: 20
                    color: Theme.error
                }

                Label {
                    text: "Remove Project"
                    font.pixelSize: Theme.fontSizeMedium
                    font.bold: true
                    color: Theme.error
                }
            }
        }

        onAccepted: {
            if (projectIndex >= 0) {
                projectModel.remove_project(projectIndex);
            }
            projectIndex = -1;
            projectName = "";
        }

        onRejected: {
            projectIndex = -1;
            projectName = "";
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Are you sure you want to remove this project?"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            Label {
                text: removeDialog.projectName
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                elide: Text.ElideMiddle
            }

            Label {
                text: "This will remove the project and all its tasks from the local database."
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }
    }

    Component.onCompleted: {
        projectModel.check_auth();
        // Fetch is triggered by onAuthenticatedChanged when authenticated becomes true
    }
}
