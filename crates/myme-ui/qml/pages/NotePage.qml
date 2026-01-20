import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Page {
    id: notePage
    title: "Notes"

    header: ToolBar {
        RowLayout {
            anchors.fill: parent
            spacing: 10

            ToolButton {
                text: "\u21BB"  // Refresh symbol
                font.pixelSize: 20
                onClicked: noteModel.fetch_notes()
                ToolTip.text: "Refresh notes"
                ToolTip.visible: hovered
            }

            Label {
                text: "Notes"
                font.pixelSize: 18
                font.bold: true
                Layout.fillWidth: true
            }

            ToolButton {
                text: "+"
                font.pixelSize: 24
                onClicked: addDialog.open()
                ToolTip.text: "Add new note"
                ToolTip.visible: hovered
            }
        }
    }

    // Main content
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        // Error message banner
        Rectangle {
            visible: noteModel.error_message.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: 60
            color: "#FFE6E6"
            border.color: "#FF4444"
            border.width: 1
            radius: 4

            RowLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 10

                Label {
                    text: "\u26A0"  // Warning symbol
                    font.pixelSize: 20
                    color: "#FF4444"
                }

                Label {
                    text: noteModel.error_message
                    color: "#CC0000"
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                }

                Button {
                    text: "Retry"
                    onClicked: {
                        noteModel.fetch_notes()
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
                spacing: 5

                model: noteModel.row_count()

                delegate: Rectangle {
                    required property int index

                    width: notesList.width
                    height: noteContent.height + 20
                    color: index % 2 === 0 ? "#F9F9F9" : "#FFFFFF"
                    border.color: "#E0E0E0"
                    border.width: 1
                    radius: 4

                    MouseArea {
                        anchors.fill: parent
                        onClicked: noteModel.toggle_done(parent.index)
                        cursorShape: Qt.PointingHandCursor
                    }

                    RowLayout {
                        id: noteContent
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 10

                        // Status checkbox
                        CheckBox {
                            checked: noteModel.get_done(parent.parent.index)
                            onClicked: noteModel.toggle_done(parent.parent.parent.index)
                        }

                        // Note text and metadata
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 5

                            Label {
                                text: noteModel.get_content(parent.parent.parent.index)
                                font.pixelSize: 14
                                font.strikeout: noteModel.get_done(parent.parent.parent.index)
                                opacity: noteModel.get_done(parent.parent.parent.index) ? 0.6 : 1.0
                                Layout.fillWidth: true
                                wrapMode: Text.WordWrap
                            }

                            Label {
                                text: noteModel.get_created_at(parent.parent.parent.index)
                                font.pixelSize: 11
                                color: "#888888"
                            }

                            Label {
                                text: noteModel.get_done(parent.parent.parent.index) ? "✓ Completed" : "○ Pending"
                                font.pixelSize: 11
                                color: noteModel.get_done(parent.parent.parent.index) ? "#4CAF50" : "#FFA726"
                            }
                        }

                        // Action buttons
                        ColumnLayout {
                            spacing: 5

                            Button {
                                text: noteModel.get_done(parent.parent.parent.index) ? "Undo" : "Done"
                                onClicked: noteModel.toggle_done(parent.parent.parent.parent.index)
                            }

                            Button {
                                text: "Delete"
                                onClicked: noteModel.delete_note(parent.parent.parent.parent.index)
                            }
                        }
                    }
                }

                // Empty state
                Label {
                    visible: !noteModel.loading && noteModel.row_count() === 0
                    anchors.centerIn: parent
                    text: "No notes yet\n\nClick + to add your first note"
                    font.pixelSize: 16
                    color: "#888888"
                    horizontalAlignment: Text.AlignHCenter
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
            Layout.preferredHeight: 40
            color: "#F0F0F0"
            border.color: "#E0E0E0"
            border.width: 1
            radius: 4

            RowLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 10

                Label {
                    text: {
                        var total = noteModel.row_count()
                        var done = 0
                        for (var i = 0; i < total; i++) {
                            if (noteModel.get_done(i)) done++
                        }
                        return total + " notes (" + done + " done, " + (total - done) + " pending)"
                    }
                    font.pixelSize: 12
                }

                Item { Layout.fillWidth: true }

                Label {
                    text: "● Godo API Connected"
                    font.pixelSize: 11
                    color: "#4CAF50"
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
        height: 300

        onAccepted: {
            if (contentField.text.trim().length > 0) {
                noteModel.add_note(contentField.text)
                contentField.text = ""
            }
        }

        onRejected: {
            contentField.text = ""
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: 10

            Label {
                text: "Note content:"
            }

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true

                TextArea {
                    id: contentField
                    placeholderText: "Enter note content (1-1000 characters)..."
                    wrapMode: TextEdit.Wrap
                }
            }

            Label {
                text: contentField.text.length + " / 1000 characters"
                font.pixelSize: 11
                color: contentField.text.length > 1000 ? "#FF4444" : "#888888"
            }

            Label {
                text: "Tip: Press Ctrl+Enter to save quickly"
                font.pixelSize: 10
                color: "#888888"
            }
        }

        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: addDialog.accept()
        }
    }

    Component.onCompleted: {
        noteModel.fetch_notes()
    }
}
