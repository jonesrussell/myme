import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: projectDetailPage
    title: projectName

    required property string projectId
    required property string projectName

    readonly property bool hasRepos: {
        try {
            const repos = JSON.parse(kanbanModel.repo_ids);
            return repos && repos.length > 0;
        } catch (e) { return false; }
    }

    // Kanban columns configuration
    readonly property var columns: [
        { key: "backlog", label: "Backlog", color: "#8a8580" },
        { key: "todo", label: "Todo", color: "#64b5f6" },
        { key: "inprogress", label: "In Progress", color: "#e5a54b" },
        { key: "blocked", label: "Blocked", color: "#e57373" },
        { key: "review", label: "Review", color: "#b39ddb" },
        { key: "done", label: "Done", color: "#5bb98c" }
    ]

    // Track the currently dragged task
    property int draggedTaskIndex: -1
    property string draggedFromColumn: ""

    background: Rectangle {
        color: Theme.background
    }

    // Models
    ProjectModel {
        id: projectModel
    }

    KanbanModel {
        id: kanbanModel
    }

    // Poll timer for async project operations (add repo)
    Timer {
        id: projectPollTimer
        interval: 100
        running: projectModel.loading
        repeat: true
        onTriggered: projectModel.poll_channel()
    }

    // Poll timer for async kanban operations
    Timer {
        id: kanbanPollTimer
        interval: 100
        running: kanbanModel.loading
        repeat: true
        onTriggered: kanbanModel.poll_channel()
    }

    // Force UI update when loading finishes
    Connections {
        target: kanbanModel
        function onLoadingChanged() {
            if (!kanbanModel.loading) {
                columnsRepeater.model = 0;
                columnsRepeater.model = projectDetailPage.columns.length;
            }
        }
    }

    // Reload kanban when project repos change (e.g. after adding repo)
    Connections {
        target: projectModel
        function onProjects_changed() {
            kanbanModel.load_project(projectId);
        }
    }

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            // Back button
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
                text: projectName
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
                elide: Text.ElideMiddle
            }

            // Add repo button
            ToolButton {
                text: Icons.plus
                font.family: Icons.family
                font.pixelSize: 18
                enabled: !projectModel.loading
                onClicked: addRepoDialog.open()
                ToolTip.text: "Add Repo"
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

            // Sync button
            ToolButton {
                text: Icons.arrowsClockwise
                font.family: Icons.family
                font.pixelSize: 18
                enabled: !kanbanModel.loading
                onClicked: kanbanModel.sync_tasks()
                ToolTip.text: "Sync with GitHub"
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

            // New task button (disabled when no repos - add repo first)
            ToolButton {
                text: Icons.plus
                font.family: Icons.family
                font.pixelSize: 18
                enabled: projectDetailPage.hasRepos && !kanbanModel.loading
                onClicked: newTaskDialog.open()
                ToolTip.text: projectDetailPage.hasRepos ? "New Task" : "Add a repo first to create tasks"
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
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingMd

        // Error message banner
        Rectangle {
            visible: kanbanModel.error_message.length > 0
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
                    text: kanbanModel.error_message
                    font.family: Theme.fontFamily
                    color: Theme.error
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                }

                Button {
                    text: "Retry"
                    onClicked: kanbanModel.load_project(projectId)

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

        // Loading indicator (centered when no tasks)
        BusyIndicator {
            visible: kanbanModel.loading && kanbanModel.row_count() === 0
            running: kanbanModel.loading
            Layout.alignment: Qt.AlignHCenter
        }

        // Kanban board
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AsNeeded
            ScrollBar.vertical.policy: ScrollBar.AlwaysOff

            RowLayout {
                id: kanbanBoard
                height: parent.height
                spacing: Theme.spacingMd

                Repeater {
                    id: columnsRepeater
                    model: projectDetailPage.columns.length

                    delegate: Rectangle {
                        id: columnContainer
                        required property int index

                        property var columnData: projectDetailPage.columns[index]
                        property string columnKey: columnData.key
                        property string columnLabel: columnData.label
                        property color columnColor: columnData.color

                        Layout.preferredWidth: 280
                        Layout.fillHeight: true
                        Layout.minimumHeight: 400
                        color: Theme.surface
                        border.color: dropArea.containsDrag ? Theme.primary : Theme.border
                        border.width: dropArea.containsDrag ? 2 : 1
                        radius: Theme.cardRadius

                        Behavior on border.color {
                            ColorAnimation { duration: 100 }
                        }

                        // Drop area for drag and drop
                        DropArea {
                            id: dropArea
                            anchors.fill: parent
                            keys: ["task"]

                            onDropped: (drop) => {
                                if (projectDetailPage.draggedTaskIndex >= 0) {
                                    kanbanModel.move_task(projectDetailPage.draggedTaskIndex, columnContainer.columnKey);
                                    projectDetailPage.draggedTaskIndex = -1;
                                    projectDetailPage.draggedFromColumn = "";
                                }
                            }
                        }

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingSm
                            spacing: Theme.spacingSm

                            // Column header
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm

                                // Colored indicator
                                Rectangle {
                                    width: 4
                                    height: 20
                                    radius: 2
                                    color: columnContainer.columnColor
                                }

                                Label {
                                    text: columnContainer.columnLabel
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeMedium
                                    font.bold: true
                                    color: Theme.text
                                    Layout.fillWidth: true
                                }

                                // Count badge
                                Rectangle {
                                    visible: kanbanModel.count_by_status(columnContainer.columnKey) > 0
                                    width: countLabel.implicitWidth + Theme.spacingSm * 2
                                    height: 22
                                    radius: 11
                                    color: columnContainer.columnColor + "30"

                                    Label {
                                        id: countLabel
                                        anchors.centerIn: parent
                                        text: kanbanModel.count_by_status(columnContainer.columnKey)
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.bold: true
                                        color: columnContainer.columnColor
                                    }
                                }

                                // Add task button for this column
                                Rectangle {
                                    width: 24
                                    height: 24
                                    radius: Theme.buttonRadius
                                    color: addMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                                    Label {
                                        anchors.centerIn: parent
                                        text: Icons.plus
                                        font.family: Icons.family
                                        font.pixelSize: 14
                                        color: Theme.textSecondary
                                    }

                                    MouseArea {
                                        id: addMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        enabled: projectDetailPage.hasRepos
                                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                                        onClicked: {
                                            if (projectDetailPage.hasRepos) {
                                                newTaskDialog.preselectedStatus = columnContainer.columnKey;
                                                newTaskDialog.open();
                                            }
                                        }
                                    }

                                    ToolTip.visible: addMouseArea.containsMouse
                                    ToolTip.text: projectDetailPage.hasRepos ? ("Add task to " + columnContainer.columnLabel) : "Add a repo first"
                                    ToolTip.delay: 500
                                }
                            }

                            // Separator
                            Rectangle {
                                Layout.fillWidth: true
                                height: 1
                                color: Theme.borderLight
                            }

                            // Tasks list
                            ListView {
                                id: tasksList
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                clip: true
                                spacing: Theme.spacingSm

                                model: {
                                    try {
                                        const indices = JSON.parse(kanbanModel.tasks_for_status(columnContainer.columnKey));
                                        return indices;
                                    } catch (e) {
                                        return [];
                                    }
                                }

                                delegate: Rectangle {
                                    id: taskCard
                                    required property int modelData
                                    property int taskIndex: modelData

                                    width: tasksList.width
                                    height: taskContent.implicitHeight + Theme.spacingMd * 2
                                    color: taskMouseArea.containsMouse ? Theme.surfaceHover : Theme.surfaceAlt
                                    border.color: taskMouseArea.containsMouse ? Theme.primary : Theme.borderLight
                                    border.width: 1
                                    radius: Theme.cardRadius
                                    opacity: dragHandler.active ? 0.8 : 1.0

                                    Behavior on color {
                                        ColorAnimation { duration: 100 }
                                    }
                                    Behavior on border.color {
                                        ColorAnimation { duration: 100 }
                                    }

                                    // Drag handling
                                    Drag.active: dragHandler.active
                                    Drag.keys: ["task"]
                                    Drag.hotSpot.x: width / 2
                                    Drag.hotSpot.y: height / 2

                                    DragHandler {
                                        id: dragHandler
                                        onActiveChanged: {
                                            if (active) {
                                                projectDetailPage.draggedTaskIndex = taskCard.taskIndex;
                                                projectDetailPage.draggedFromColumn = columnContainer.columnKey;
                                                taskCard.z = 100;
                                            } else {
                                                taskCard.z = 0;
                                            }
                                        }
                                    }

                                    MouseArea {
                                        id: taskMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        // Let DragHandler handle dragging
                                        onClicked: {
                                            taskDetailDialog.taskIndex = taskCard.taskIndex;
                                            taskDetailDialog.taskTitle = kanbanModel.get_title(taskCard.taskIndex);
                                            taskDetailDialog.taskBody = kanbanModel.get_body(taskCard.taskIndex);
                                            taskDetailDialog.open();
                                        }
                                    }

                                    ColumnLayout {
                                        id: taskContent
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.top: parent.top
                                        anchors.margins: Theme.spacingMd
                                        spacing: Theme.spacingXs

                                        // Issue number and GitHub link
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: Theme.spacingXs

                                            Label {
                                                text: "#" + kanbanModel.get_issue_number(taskCard.taskIndex)
                                                font.family: Theme.fontFamily
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.bold: true
                                                color: columnContainer.columnColor
                                            }

                                            Item { Layout.fillWidth: true }

                                            // Open on GitHub button
                                            Rectangle {
                                                width: 20
                                                height: 20
                                                radius: 4
                                                color: githubMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                                                Label {
                                                    anchors.centerIn: parent
                                                    text: Icons.githubLogo
                                                    font.family: Icons.family
                                                    font.pixelSize: 12
                                                    color: Theme.textSecondary
                                                }

                                                MouseArea {
                                                    id: githubMouseArea
                                                    anchors.fill: parent
                                                    hoverEnabled: true
                                                    cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        Qt.openUrlExternally(kanbanModel.get_url(taskCard.taskIndex));
                                                    }
                                                }

                                                ToolTip.visible: githubMouseArea.containsMouse
                                                ToolTip.text: "Open on GitHub"
                                                ToolTip.delay: 500
                                            }
                                        }

                                        // Repo badge (when multiple repos)
                                        Label {
                                            visible: {
                                                try {
                                                    const repos = JSON.parse(kanbanModel.repo_ids);
                                                    return repos && repos.length > 1;
                                                } catch (e) { return false; }
                                            }
                                            text: kanbanModel.get_repo_id(taskCard.taskIndex)
                                            font.family: Theme.fontFamily
                                            font.pixelSize: Theme.fontSizeSmall - 1
                                            color: Theme.textMuted
                                            Layout.fillWidth: true
                                            elide: Text.ElideMiddle
                                        }

                                        // Task title
                                        Label {
                                            text: kanbanModel.get_title(taskCard.taskIndex)
                                            font.family: Theme.fontFamily
                                            font.pixelSize: Theme.fontSizeNormal
                                            color: Theme.text
                                            Layout.fillWidth: true
                                            wrapMode: Text.WordWrap
                                            maximumLineCount: 3
                                            elide: Text.ElideRight
                                        }
                                    }
                                }
                            }

                            // Empty column placeholder
                            Item {
                                visible: kanbanModel.count_by_status(columnContainer.columnKey) === 0
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                Label {
                                    anchors.centerIn: parent
                                    text: "No tasks"
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textMuted
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // New task dialog
    Dialog {
        id: newTaskDialog
        title: "New Task"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 500)
        height: 350

        property string preselectedStatus: "todo"

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
                text: "New Task"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
        }

        onAccepted: {
            if (newTaskTitleField.text.trim().length > 0) {
                const statusKey = projectDetailPage.columns[newTaskStatusCombo.currentIndex].key;
                const repoId = newTaskRepoCombo.currentIndex >= 0 && newTaskRepoCombo.count > 0
                    ? newTaskRepoCombo.currentText
                    : "";
                kanbanModel.create_task(
                    newTaskTitleField.text.trim(),
                    newTaskDescField.text.trim(),
                    statusKey,
                    repoId
                );
                newTaskTitleField.text = "";
                newTaskDescField.text = "";
            }
        }

        onRejected: {
            newTaskTitleField.text = "";
            newTaskDescField.text = "";
        }

        onOpened: {
            // Find index for preselected status
            for (let i = 0; i < projectDetailPage.columns.length; i++) {
                if (projectDetailPage.columns[i].key === preselectedStatus) {
                    newTaskStatusCombo.currentIndex = i;
                    break;
                }
            }
            newTaskTitleField.forceActiveFocus();
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Title:"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: Theme.inputBg
                border.color: newTaskTitleField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: newTaskTitleField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                TextField {
                    id: newTaskTitleField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "Task title"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }

            Label {
                text: "Description:"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 80
                color: Theme.inputBg
                border.color: newTaskDescField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: newTaskDescField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 2

                    TextArea {
                        id: newTaskDescField
                        placeholderText: "Task description (optional)"
                        color: Theme.text
                        placeholderTextColor: Theme.textMuted
                        wrapMode: TextArea.Wrap

                        background: Rectangle {
                            color: "transparent"
                        }
                    }
                }
            }

            Label {
                text: "Repo (if multiple):"
                visible: newTaskRepoCombo.count > 1
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            ComboBox {
                id: newTaskRepoCombo
                visible: count > 1
                Layout.fillWidth: true
                model: {
                    try {
                        const repos = JSON.parse(kanbanModel.repo_ids);
                        return repos || [];
                    } catch (e) {
                        return [];
                    }
                }

                background: Rectangle {
                    color: Theme.inputBg
                    border.color: newTaskRepoCombo.pressed ? Theme.primary : Theme.inputBorder
                    border.width: 1
                    radius: Theme.inputRadius
                }

                contentItem: Text {
                    leftPadding: Theme.spacingSm
                    text: newTaskRepoCombo.displayText
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.text
                    verticalAlignment: Text.AlignVCenter
                }

                delegate: ItemDelegate {
                    width: newTaskRepoCombo.width
                    contentItem: Text {
                        text: modelData
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.text
                    }
                    background: Rectangle {
                        color: highlighted ? Theme.surfaceHover : Theme.surface
                    }
                }

                popup: Popup {
                    y: newTaskRepoCombo.height
                    width: newTaskRepoCombo.width
                    implicitHeight: contentItem.implicitHeight
                    padding: 1
                    contentItem: ListView {
                        clip: true
                        implicitHeight: contentHeight
                        model: newTaskRepoCombo.popup.visible ? newTaskRepoCombo.delegateModel : null
                        currentIndex: newTaskRepoCombo.highlightedIndex
                    }
                    background: Rectangle {
                        color: Theme.surface
                        border.color: Theme.border
                        border.width: 1
                        radius: Theme.inputRadius
                    }
                }
            }

            Label {
                text: "Status:"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            ComboBox {
                id: newTaskStatusCombo
                Layout.fillWidth: true
                model: projectDetailPage.columns.map(col => col.label)
                currentIndex: 1 // Default to "Todo"

                background: Rectangle {
                    color: Theme.inputBg
                    border.color: newTaskStatusCombo.pressed ? Theme.primary : Theme.inputBorder
                    border.width: 1
                    radius: Theme.inputRadius
                }

                contentItem: Text {
                    leftPadding: Theme.spacingSm
                    text: newTaskStatusCombo.displayText
                    font.pixelSize: Theme.fontSizeNormal
                    color: Theme.text
                    verticalAlignment: Text.AlignVCenter
                }

                delegate: ItemDelegate {
                    width: newTaskStatusCombo.width
                    contentItem: RowLayout {
                        spacing: Theme.spacingSm

                        Rectangle {
                            width: 8
                            height: 8
                            radius: 4
                            color: projectDetailPage.columns[index].color
                        }

                        Text {
                            text: modelData
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                        }
                    }

                    background: Rectangle {
                        color: highlighted ? Theme.surfaceHover : Theme.surface
                    }
                }

                popup: Popup {
                    y: newTaskStatusCombo.height
                    width: newTaskStatusCombo.width
                    implicitHeight: contentItem.implicitHeight
                    padding: 1

                    contentItem: ListView {
                        clip: true
                        implicitHeight: contentHeight
                        model: newTaskStatusCombo.popup.visible ? newTaskStatusCombo.delegateModel : null
                        currentIndex: newTaskStatusCombo.highlightedIndex
                    }

                    background: Rectangle {
                        color: Theme.surface
                        border.color: Theme.border
                        border.width: 1
                        radius: Theme.inputRadius
                    }
                }
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: newTaskDialog.accept()
        }
    }

    // Task detail dialog
    Dialog {
        id: taskDetailDialog
        title: "Edit Task"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 500)
        height: 320

        property int taskIndex: -1
        property string taskTitle: ""
        property string taskBody: ""

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

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacingMd
                anchors.rightMargin: Theme.spacingMd
                spacing: Theme.spacingSm

                Label {
                    text: "Edit Task"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeMedium
                    font.bold: true
                    color: Theme.text
                    Layout.fillWidth: true
                }

                // Open on GitHub button in dialog header
                Rectangle {
                    width: 28
                    height: 28
                    radius: Theme.buttonRadius
                    color: dialogGithubMouseArea.containsMouse ? Theme.surfaceHover : "transparent"

                    Label {
                        anchors.centerIn: parent
                        text: Icons.githubLogo
                        font.family: Icons.family
                        font.pixelSize: 16
                        color: Theme.textSecondary
                    }

                    MouseArea {
                        id: dialogGithubMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            Qt.openUrlExternally(kanbanModel.get_url(taskDetailDialog.taskIndex));
                        }
                    }

                    ToolTip.visible: dialogGithubMouseArea.containsMouse
                    ToolTip.text: "Open on GitHub"
                    ToolTip.delay: 500
                }
            }
        }

        onAccepted: {
            if (editTitleField.text.trim().length > 0) {
                kanbanModel.update_task(
                    taskIndex,
                    editTitleField.text.trim(),
                    editBodyField.text.trim()
                );
            }
        }

        onOpened: {
            editTitleField.text = taskTitle;
            editBodyField.text = taskBody;
            editTitleField.forceActiveFocus();
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Title:"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: Theme.inputBg
                border.color: editTitleField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: editTitleField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                TextField {
                    id: editTitleField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "Task title"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }

            Label {
                text: "Description:"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: Theme.inputBg
                border.color: editBodyField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: editBodyField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 2

                    TextArea {
                        id: editBodyField
                        placeholderText: "Task description"
                        color: Theme.text
                        placeholderTextColor: Theme.textMuted
                        wrapMode: TextArea.Wrap

                        background: Rectangle {
                            color: "transparent"
                        }
                    }
                }
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: taskDetailDialog.accept()
        }
    }

    // Add repo dialog
    Dialog {
        id: addRepoDialog
        title: "Add Repo to Project"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 450)
        height: 200

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
                text: "Add Repo"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
        }

        onAccepted: {
            if (addRepoField.text.trim().length > 0) {
                projectModel.add_repo_to_project_by_id(projectId, addRepoField.text.trim());
                addRepoField.text = "";
            }
        }

        onRejected: {
            addRepoField.text = "";
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "GitHub repository (owner/repo):"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 44
                color: Theme.inputBg
                border.color: addRepoField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: addRepoField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                TextField {
                    id: addRepoField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "owner/repo"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: addRepoDialog.accept()
        }
    }

    Component.onCompleted: {
        projectModel.check_auth();
        kanbanModel.load_project(projectId);
    }
}
