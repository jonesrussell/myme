import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: notePage
    title: "Notes"

    // Track note count for ListView model
    property int noteCount: 0

    background: Rectangle {
        color: Theme.background
    }

    // Instantiate the NoteModel from Rust
    NoteModel {
        id: noteModel
    }

    // Timer to poll for async operation results (non-blocking)
    Timer {
        id: pollTimer
        interval: 100  // Poll every 100ms
        running: true  // Always running to check for results
        repeat: true
        onTriggered: noteModel.poll_channel()
    }

    // Update note count when notes change
    Connections {
        target: noteModel
        function onNotes_changed() {
            notePage.noteCount = noteModel.row_count();
        }
        function onLoadingChanged() {
            if (!noteModel.loading) {
                notePage.noteCount = noteModel.row_count();
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
                text: Icons.arrowsClockwise
                font.family: Icons.family
                font.pixelSize: 18
                onClicked: noteModel.fetch_notes()
                ToolTip.text: "Refresh notes"
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
                text: "Notes"
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.text
                Layout.fillWidth: true
            }

            ToolButton {
                text: Icons.plus
                font.family: Icons.family
                font.pixelSize: 18
                onClicked: addDialog.open()
                ToolTip.text: "Add new note"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.primary : Theme.surfaceHover
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: parent.parent.hovered ? Theme.primaryText : Theme.text
                    font.pixelSize: 18
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }
    }

    // Main content
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        spacing: Theme.spacingMd

        // Error message banner
        Rectangle {
            visible: noteModel.error_message.length > 0
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
                    text: noteModel.error_message
                    color: Theme.error
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                }

                Button {
                    text: "Retry"
                    onClicked: noteModel.fetch_notes()

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

        // Notes list
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: notesList
                anchors.fill: parent
                spacing: Theme.spacingSm

                model: notePage.noteCount

                delegate: Rectangle {
                    id: noteDelegate
                    required property int index

                    width: notesList.width
                    height: noteContent.implicitHeight + Theme.spacingMd * 2
                    color: noteDelegateMouseArea.containsMouse ? Theme.surfaceHover : Theme.surface
                    border.color: Theme.border
                    border.width: 1
                    radius: Theme.cardRadius

                    Behavior on color {
                        ColorAnimation {
                            duration: 100
                        }
                    }

                    MouseArea {
                        id: noteDelegateMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: noteModel.toggle_done(noteDelegate.index)
                        cursorShape: Qt.PointingHandCursor
                    }

                    RowLayout {
                        id: noteContent
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: Theme.spacingMd
                        spacing: Theme.spacingMd

                        // Status checkbox
                        CheckBox {
                            checked: noteModel.get_done(noteDelegate.index)
                            onClicked: noteModel.toggle_done(noteDelegate.index)
                        }

                        // Note text and metadata
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacingXs

                            Label {
                                text: noteModel.get_content(noteDelegate.index)
                                font.pixelSize: Theme.fontSizeNormal
                                font.strikeout: noteModel.get_done(noteDelegate.index)
                                color: Theme.text
                                opacity: noteModel.get_done(noteDelegate.index) ? 0.6 : 1.0
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                            }

                            Label {
                                text: noteModel.get_created_at(noteDelegate.index)
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.textMuted
                            }

                            Rectangle {
                                width: statusLabel.implicitWidth + Theme.spacingSm * 2
                                height: statusLabel.implicitHeight + Theme.spacingXs
                                radius: 4
                                color: noteModel.get_done(noteDelegate.index) ? Theme.success + "20" : Theme.warning + "20"

                                Label {
                                    id: statusLabel
                                    anchors.centerIn: parent
                                    text: noteModel.get_done(noteDelegate.index) ? "Completed" : "Pending"
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.bold: true
                                    color: noteModel.get_done(noteDelegate.index) ? Theme.success : Theme.warning
                                }
                            }
                        }

                        // Action buttons
                        ColumnLayout {
                            spacing: Theme.spacingXs

                            Button {
                                text: noteModel.get_done(noteDelegate.index) ? "Undo" : "Done"
                                onClicked: noteModel.toggle_done(noteDelegate.index)

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.surfaceHover : Theme.surfaceAlt
                                    border.color: Theme.border
                                    border.width: 1
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.text
                                    font.pixelSize: Theme.fontSizeSmall
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                text: "Delete"
                                onClicked: noteModel.delete_note(noteDelegate.index)

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.error + "30" : "transparent"
                                    border.color: Theme.error
                                    border.width: 1
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: Theme.error
                                    font.pixelSize: Theme.fontSizeSmall
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                text: Icons.squaresFour
                                font.family: Icons.family
                                onClicked: {
                                    promoteDialog.noteIndex = noteDelegate.index;
                                    promoteDialog.noteTitle = noteModel.get_content(noteDelegate.index);
                                    promoteDialog.open();
                                }
                                ToolTip.text: "Promote to Project"
                                ToolTip.visible: hovered

                                background: Rectangle {
                                    radius: Theme.buttonRadius
                                    color: parent.hovered ? Theme.primary + "30" : "transparent"
                                    border.color: Theme.primary
                                    border.width: 1
                                }

                                contentItem: Text {
                                    text: parent.text
                                    font.family: Icons.family
                                    color: Theme.primary
                                    font.pixelSize: Theme.fontSizeSmall
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }
                        }
                    }
                }

                // Empty state
                Column {
                    visible: !noteModel.loading && noteModel.row_count() === 0
                    anchors.centerIn: parent
                    spacing: Theme.spacingMd

                    Label {
                        text: Icons.notePencil
                        font.family: Icons.family
                        font.pixelSize: 48
                        color: Theme.textMuted
                        anchors.horizontalCenter: parent.horizontalCenter
                    }

                    Label {
                        text: "No notes yet"
                        font.pixelSize: Theme.fontSizeLarge
                        font.bold: true
                        color: Theme.text
                        anchors.horizontalCenter: parent.horizontalCenter
                    }

                    Label {
                        text: "Click + to add your first note"
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                }
            }
        }

        // Loading indicator
        BusyIndicator {
            visible: noteModel.loading
            running: noteModel.loading
            Layout.alignment: Qt.AlignHCenter
        }

        // Footer with statistics
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: Theme.surfaceAlt
            border.color: Theme.border
            border.width: 1
            radius: Theme.cardRadius

            RowLayout {
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                spacing: Theme.spacingMd

                Label {
                    text: {
                        const total = noteModel.row_count();
                        let done = 0;
                        for (let i = 0; i < total; i++) {
                            if (noteModel.get_done(i))
                                done++;
                        }
                        return `${total} notes (${done} done, ${total - done} pending)`;
                    }
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                }

                Item {
                    Layout.fillWidth: true
                }

                Rectangle {
                    width: connectedLabel.implicitWidth + Theme.spacingMd
                    height: connectedLabel.implicitHeight + Theme.spacingXs
                    radius: 4
                    color: noteModel.loading ? Theme.warning + "20" :
                           noteModel.connected ? Theme.success + "20" : Theme.error + "20"

                    Label {
                        id: connectedLabel
                        anchors.centerIn: parent
                        text: noteModel.loading ? "● Connecting..." :
                              noteModel.connected ? "● Godo API Connected" : "● Godo API Disconnected"
                        font.pixelSize: Theme.fontSizeSmall
                        color: noteModel.loading ? Theme.warning :
                               noteModel.connected ? Theme.success : Theme.error
                    }
                }
            }
        }
    }

    // Add note dialog
    Dialog {
        id: addDialog
        title: "Add New Note"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 500)
        height: 320

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

            // Cover bottom radius
            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: Theme.cardRadius
                color: Theme.surfaceAlt
            }

            Label {
                anchors.centerIn: parent
                text: "Add New Note"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
        }

        onAccepted: {
            if (contentField.text.trim().length > 0) {
                noteModel.add_note(contentField.text);
                contentField.text = "";
            }
        }

        onRejected: {
            contentField.text = "";
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Note content:"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: Theme.inputBg
                border.color: contentField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: contentField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                Behavior on border.color {
                    ColorAnimation {
                        duration: 100
                    }
                }

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 2

                    TextArea {
                        id: contentField
                        placeholderText: "Enter note content (1-1000 characters)..."
                        wrapMode: TextEdit.Wrap
                        color: Theme.text
                        placeholderTextColor: Theme.textMuted

                        background: Rectangle {
                            color: "transparent"
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: contentField.text.length + " / 1000 characters"
                    font.pixelSize: Theme.fontSizeSmall
                    color: contentField.text.length > 1000 ? Theme.error : Theme.textMuted
                }

                Item {
                    Layout.fillWidth: true
                }

                Label {
                    text: "Ctrl+Enter to save"
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textMuted
                }
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: addDialog.accept()
        }
    }

    Component.onCompleted: {
        noteModel.fetch_notes();
    }

    // Promote to Project dialog
    Dialog {
        id: promoteDialog
        title: "Promote to Project"
        standardButtons: Dialog.Ok | Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 450)

        property int noteIndex: -1
        property string noteTitle: ""

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
                text: "Promote to Project"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
        }

        onAccepted: {
            if (repoNameField.text.trim().length > 0) {
                // Navigate to projects page
                // Note: Full integration would add the project via ProjectModel
                stackView.replace("ProjectsPage.qml");
            }
        }

        onRejected: {
            repoNameField.text = "";
        }

        onOpened: {
            repoNameField.text = "";
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Create a new project from this note?"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: notePreview.implicitHeight + Theme.spacingMd * 2
                color: Theme.surfaceAlt
                border.color: Theme.border
                radius: Theme.cardRadius

                Label {
                    id: notePreview
                    anchors.fill: parent
                    anchors.margins: Theme.spacingMd
                    text: promoteDialog.noteTitle
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textSecondary
                    wrapMode: Text.WordWrap
                    elide: Text.ElideRight
                    maximumLineCount: 3
                }
            }

            Label {
                text: "GitHub Repository (owner/repo):"
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 40
                color: Theme.inputBg
                border.color: repoNameField.activeFocus ? Theme.primary : Theme.inputBorder
                border.width: repoNameField.activeFocus ? 2 : 1
                radius: Theme.inputRadius

                TextField {
                    id: repoNameField
                    anchors.fill: parent
                    anchors.margins: 2
                    placeholderText: "e.g., jonesrussell/my-project"
                    color: Theme.text
                    placeholderTextColor: Theme.textMuted

                    background: Rectangle {
                        color: "transparent"
                    }
                }
            }

            Label {
                text: "The note will be used as the project description."
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: promoteDialog.accept()
        }
    }
}
