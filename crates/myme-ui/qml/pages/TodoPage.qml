import QtQuick
import QtQuick.Controls as Controls
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import com.myme 1.0

Kirigami.ScrollablePage {
    id: todoPage
    title: "Todos"

    actions: [
        Kirigami.Action {
            text: "Refresh"
            icon.name: "view-refresh"
            onTriggered: todoModel.fetchTodos()
        },
        Kirigami.Action {
            text: "Add Todo"
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

                // Status indicator
                Rectangle {
                    width: 12
                    height: 12
                    radius: 6
                    color: {
                        var status = todoModel.getStatus(todoItem.index)
                        if (status === 2) return Kirigami.Theme.positiveTextColor      // completed
                        if (status === 1) return Kirigami.Theme.neutralTextColor       // in progress
                        return Kirigami.Theme.textColor                                // pending
                    }
                }

                // Todo content
                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Kirigami.Units.smallSpacing

                    Controls.Label {
                        text: todoModel.getTitle(todoItem.index)
                        font.weight: Font.Bold
                        Layout.fillWidth: true
                    }

                    Controls.Label {
                        text: todoModel.getDescription(todoItem.index)
                        font: Kirigami.Theme.smallFont
                        color: Kirigami.Theme.disabledTextColor
                        Layout.fillWidth: true
                        visible: text.length > 0
                    }

                    Controls.Label {
                        text: {
                            var status = todoModel.getStatus(todoItem.index)
                            if (status === 2) return "Completed"
                            if (status === 1) return "In Progress"
                            return "Pending"
                        }
                        font: Kirigami.Theme.smallFont
                        color: {
                            var status = todoModel.getStatus(todoItem.index)
                            if (status === 2) return Kirigami.Theme.positiveTextColor
                            if (status === 1) return Kirigami.Theme.neutralTextColor
                            return Kirigami.Theme.textColor
                        }
                    }
                }
            }

            actions: [
                Kirigami.Action {
                    text: "Complete"
                    icon.name: "dialog-ok"
                    visible: todoModel.getStatus(todoItem.index) !== 2
                    onTriggered: todoModel.completeTodo(todoItem.index)
                },
                Kirigami.Action {
                    text: "Delete"
                    icon.name: "delete"
                    onTriggered: todoModel.deleteTodo(todoItem.index)
                }
            ]
        }

        Kirigami.PlaceholderMessage {
            anchors.centerIn: parent
            width: parent.width - (Kirigami.Units.largeSpacing * 4)
            visible: !todoModel.loading && todoModel.rowCount() === 0

            icon.name: "view-task"
            text: "No todos yet"
            explanation: "Add your first todo to get started"

            helpfulAction: Kirigami.Action {
                text: "Add Todo"
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
    }

    // Add todo dialog
    Controls.Dialog {
        id: addDialog
        title: "Add New Todo"
        standardButtons: Controls.Dialog.Ok | Controls.Dialog.Cancel
        modal: true

        anchors.centerIn: parent
        width: Math.min(parent.width * 0.8, 500)

        onAccepted: {
            todoModel.addTodo(titleField.text, descriptionField.text)
            titleField.text = ""
            descriptionField.text = ""
        }

        ColumnLayout {
            width: parent.width
            spacing: Kirigami.Units.largeSpacing

            Controls.Label {
                text: "Title:"
            }

            Controls.TextField {
                id: titleField
                Layout.fillWidth: true
                placeholderText: "Enter todo title..."
            }

            Controls.Label {
                text: "Description:"
            }

            Controls.TextArea {
                id: descriptionField
                Layout.fillWidth: true
                Layout.preferredHeight: 100
                placeholderText: "Enter description (optional)..."
            }
        }
    }
}
