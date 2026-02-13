import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."
import "../components"

Page {
    id: notePage
    title: "Notes"

    property int noteCount: 0

    background: Rectangle {
        color: Theme.background
    }

    NoteModel {
        id: noteModel
    }

    Timer {
        id: pollTimer
        interval: 100
        running: true
        repeat: true
        onTriggered: noteModel.poll_channel()
    }

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

            RowLayout {
                spacing: Theme.spacingXs

                ToolButton {
                    text: "All"
                    font.pixelSize: Theme.fontSizeSmall
                    onClicked: noteModel.set_filter("all")
                    ToolTip.text: "All notes"
                    ToolTip.visible: hovered

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : "transparent"
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.text
                        font.pixelSize: Theme.fontSizeSmall
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                ToolButton {
                    text: "Archived"
                    font.pixelSize: Theme.fontSizeSmall
                    onClicked: noteModel.set_filter("archived")
                    ToolTip.text: "Archived notes"
                    ToolTip.visible: hovered

                    background: Rectangle {
                        radius: Theme.buttonRadius
                        color: parent.hovered ? Theme.surfaceHover : "transparent"
                    }

                    contentItem: Text {
                        text: parent.text
                        color: Theme.text
                        font.pixelSize: Theme.fontSizeSmall
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        spacing: Theme.spacingMd

        Rectangle {
            visible: noteModel.error_message.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: 60
            color: Theme.errorBg
            border.color: "transparent"
            border.width: 0
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

        QuickAddBar {
            id: quickAdd
            Layout.fillWidth: true
            onCreateNote: (text, isChecklist) => {
                if (text.trim().length > 0) {
                    if (isChecklist) {
                        noteModel.add_note_checklist(text);
                    } else {
                        noteModel.add_note(text);
                    }
                }
            }
        }

        ScrollView {
            id: notesScroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: notesFlow.width
            contentHeight: notesFlow.height

            Flow {
                id: notesFlow
                width: notesScroll.viewport ? notesScroll.viewport.width : notesScroll.width
                spacing: Theme.spacingMd

                Repeater {
                    model: notePage.noteCount

                    delegate: NoteCard {
                        required property int index
                        noteIndex: index
                        noteModel: noteModel
                        cardWidth: Math.min(280, Math.max(180, (notesFlow.width - Theme.spacingMd * 2) / 3))
                        onPromoteRequested: (idx, title) => {
                            promoteDialog.noteIndex = idx;
                            promoteDialog.noteTitle = title;
                            promoteDialog.open();
                        }
                    }
                }

                Column {
                    visible: !noteModel.loading && noteModel.row_count() === 0
                    width: notesFlow.width
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
                        text: "Take a note above or click + to add"
                        font.pixelSize: Theme.fontSizeNormal
                        color: Theme.textSecondary
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                }
            }
        }

        BusyIndicator {
            visible: noteModel.loading
            running: noteModel.loading
            Layout.alignment: Qt.AlignHCenter
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: Theme.surfaceAlt
            border.color: Theme.isDark ? "#ffffff08" : "#00000008"
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
                        text: noteModel.loading ? "● Loading..." :
                              noteModel.connected ? "● Connected" : "● Disconnected"
                        font.pixelSize: Theme.fontSizeSmall
                        color: noteModel.loading ? Theme.warning :
                               noteModel.connected ? Theme.success : Theme.error
                    }
                }
            }
        }
    }

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
                AppContext.pageStack.push(AppContext.pageUrl("ProjectsPage"));
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

    Component.onCompleted: {
        noteModel.fetch_notes();
    }
}
