import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.myme 1.0

Kirigami.ScrollablePage {
    id: todoPage
    title: "Notes"

    actions: [
        Kirigami.Action {
            text: "Refresh"
            icon.name: "view-refresh"
            onTriggered: todoModel.fetchTodos()
        },
        Kirigami.Action {
            text: "Add Note"
            icon.name: "list-add"
            onTriggered: addDialog.open()
        }
    ]

    TodoModel {
        id: todoModel
        Component.onCompleted: {
            fetchTodos()
        }
    }

    ListView {
        id: todoList

        model: todoModel.rowCount()

        delegate: Kirigami.SwipeListItem {
            id: todoItem

            required property int index

            RowLayout {
                spacing: Kirigami.Units.largeSpacing

                // Status indicator (checkmark)
                Kirigami.Icon {
                    source: todoModel.getDone(todoItem.index) ? "checkmark" : "empty"
                    width: 24
                    height: 24
                    color: todoModel.getDone(todoItem.index) ?
                           Kirigami.Theme.positiveTextColor :
                           Kirigami.Theme.disabledTextColor
                }

                // Note content
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Kirigami.Units.smallSpacing

                    Controls.Label {
                        text: todoModel.getContent(todoItem.index)
                        font.weight: Font.Normal
                        font.strikeout: todoModel.getDone(todoItem.index)
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        opacity: todoModel.getDone(todoItem.index) ? 0.6 : 1.0
                    }

                    Controls.Label {
                        text: todoModel.getCreatedAt(todoItem.index)
                        font: Kirigami.Theme.smallFont
                        color: Kirigami.Theme.disabledTextColor
                        Layout.fillWidth: true
                        visible: text.length > 0
                    }

                    Controls.Label {
                        text: todoModel.getDone(todoItem.index) ? "Completed" : "Pending"
                        font: Kirigami.Theme.smallFont
                        color: todoModel.getDone(todoItem.index) ?
                               Kirigami.Theme.positiveTextColor :
                               Kirigami.Theme.neutralTextColor
                    }
                }
            }

            actions: [
                Kirigami.Action {
                    text: todoModel.getDone(todoItem.index) ? "Mark Undone" : "Mark Done"
                    icon.name: todoModel.getDone(todoItem.index) ? "dialog-cancel" : "dialog-ok"
                    onTriggered: todoModel.toggleDone(todoItem.index)
                },
                Kirigami.Action {
                    text: "Delete"
                    icon.name: "delete"
                    onTriggered: todoModel.deleteTodo(todoItem.index)
                }
            ]

            // Click to toggle done status
            onClicked: {
                todoModel.toggleDone(todoItem.index)
            }
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - (Kirigami.Units.largeSpacing * 4)
            visible: !todoModel.loading && todoModel.rowCount() === 0

            icon.name: "view-task"
            text: "No notes yet"
            explanation: "Add your first note to get started with Godo"

            helpfulAction: Kirigami.Action {
                text: "Add Note"
                icon.name: "list-add"
                onTriggered: addDialog.open()
            }
        }
    }

    Controls.BusyIndicator {
        anchors.centerIn: parent
        running: todoModel.loading
        visible: running
    }

    // Error message
    Kirigami.InlineMessage {
        Layout.fillWidth: true
        visible: todoModel.errorMessage.length > 0
        type: Kirigami.MessageType.Error
        text: todoModel.errorMessage

        actions: [
            Kirigami.Action {
                text: "Retry"
                icon.name: "view-refresh"
                onTriggered: {
                    todoModel.errorMessage = ""
                    todoModel.fetchTodos()
                }
            }
        ]
    }

    // Add note dialog
    Controls.Dialog {
        id: addDialog
        title: "Add New Note"
        standardButtons: Controls.Dialog.Ok | Controls.Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 500)

        onAccepted: {
            if (contentField.text.trim().length > 0) {
                todoModel.addTodo(contentField.text)
                contentField.text = ""
            }
        }

        onRejected: {
            contentField.text = ""
        }

        ColumnLayout {
            width: parent.width
            spacing: Kirigami.Units.largeSpacing

            Controls.Label {
                text: "Note content:"
            }

            Controls.TextArea {
                id: contentField
                Layout.fillWidth: true
                Layout.preferredHeight: 150
                placeholderText: "Enter note content (1-1000 characters)..."
                wrapMode: TextEdit.Wrap

                // Character count
                background: Rectangle {
                    color: Kirigami.Theme.backgroundColor
                    border.color: Kirigami.Theme.disabledTextColor
                    border.width: 1
                    radius: 3
                }
            }

            Controls.Label {
                text: contentField.text.length + " / 1000 characters"
                font: Kirigami.Theme.smallFont
                color: contentField.text.length > 1000 ?
                       Kirigami.Theme.negativeTextColor :
                       Kirigami.Theme.disabledTextColor
            }

            Controls.Label {
                text: "Tip: Press Ctrl+Enter to save quickly"
                font: Kirigami.Theme.smallFont
                color: Kirigami.Theme.disabledTextColor
                Layout.fillWidth: true
            }
        }

        // Allow Ctrl+Enter to accept
        Shortcut {
            sequence: "Ctrl+Return"
            onActivated: addDialog.accept()
        }
    }

    // Footer with statistics
    footer: Controls.ToolBar {
        RowLayout {
            anchors.fill: parent
            spacing: Kirigami.Units.largeSpacing

            Controls.Label {
                text: {
                    var total = todoModel.rowCount()
                    var done = 0
                    for (var i = 0; i < total; i++) {
                        if (todoModel.getDone(i)) done++
                    }
                    return total + " notes (" + done + " done, " + (total - done) + " pending)"
                }
                font: Kirigami.Theme.smallFont
            }

            Item { Layout.fillWidth: true }

            Controls.ToolButton {
                text: "Godo API"
                icon.name: "network-connect"
                display: Controls.AbstractButton.IconOnly
                Controls.ToolTip.text: "Connected to Godo API"
                Controls.ToolTip.visible: hovered
                flat: true
            }
        }
    }
}
