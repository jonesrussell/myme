import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: quickAddBar
    signal createNote(string text, bool isChecklist)

    implicitHeight: expanded ? addColumn.implicitHeight + Theme.spacingMd * 2 : 56
    radius: Theme.cardRadius
    color: Theme.surface
    border.color: addField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff08" : "#00000008")
    border.width: addField.activeFocus ? 2 : 1

    property bool expanded: false
    property bool isChecklist: false

    signal createNote(string text, bool isChecklist)

    Behavior on implicitHeight {
        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
    }
    Behavior on border.color {
        ColorAnimation { duration: 100 }
    }

    ColumnLayout {
        id: addColumn
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingSm

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            TextArea {
                id: addField
                Layout.fillWidth: true
                Layout.preferredHeight: expanded ? 120 : 36
                Layout.minimumHeight: 36
                placeholderText: expanded ? (isChecklist ? "List item" : "Take a note...") : "Take a note..."
                wrapMode: TextEdit.Wrap
                color: Theme.text
                placeholderTextColor: Theme.textMuted
                font.pixelSize: Theme.fontSizeNormal
                verticalAlignment: TextEdit.AlignTop
                padding: 8

                background: Rectangle {
                    color: "transparent"
                }

                Keys.onReturnPressed: event => {
                    if (!event.modifiers) {
                        if (expanded && text.trim().length > 0) {
                            quickAddBar.createNote(text, isChecklist);
                            addField.text = "";
                            expanded = false;
                            addField.focus = false;
                        }
                        event.accepted = true;
                    }
                }

                onActiveFocusChanged: {
                    if (activeFocus) {
                        expanded = true;
                    } else if (text.trim().length === 0) {
                        expanded = false;
                    } else {
                        quickAddBar.createNote(text, isChecklist);
                        addField.text = "";
                        expanded = false;
                    }
                }

                Behavior on Layout.preferredHeight {
                    NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            visible: expanded
            spacing: Theme.spacingSm

            ToolButton {
                text: isChecklist ? Icons.textT : Icons.list
                font.family: Icons.family
                font.pixelSize: 16
                onClicked: isChecklist = !isChecklist
                ToolTip.text: isChecklist ? "Note" : "New list"
                ToolTip.visible: hovered

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                }

                contentItem: Text {
                    text: parent.text
                    font.family: Icons.family
                    color: Theme.textMuted
                    font.pixelSize: 16
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }

            Item {
                Layout.fillWidth: true
            }

            ToolButton {
                text: "Close"
                font.pixelSize: Theme.fontSizeSmall
                onClicked: {
                    if (addField.text.trim().length > 0) {
                        quickAddBar.createNote(addField.text, isChecklist);
                        addField.text = "";
                    }
                    expanded = false;
                    addField.focus = false;
                }

                background: Rectangle {
                    radius: Theme.buttonRadius
                    color: parent.hovered ? Theme.surfaceHover : "transparent"
                }

                contentItem: Text {
                    text: parent.text
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeSmall
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }
    }
}
