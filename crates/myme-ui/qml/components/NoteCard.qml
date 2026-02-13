import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Rectangle {
    id: noteCard
    required property int noteIndex
    required property var noteModel

    property bool editing: false
    property bool dirty: false
    property real cardWidth: 220

    implicitWidth: cardWidth
    implicitHeight: editing ? editContent.implicitHeight + Theme.spacingMd * 2 : readContent.implicitHeight + Theme.spacingMd * 2
    radius: Theme.cardRadius
    color: {
        const c = noteModel ? noteModel.get_color(noteIndex) : "";
        return (c && c.length > 0) ? c : Theme.surface;
    }
    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
    border.width: 1

    opacity: 0
    Component.onCompleted: entryAnim.start()
    SequentialAnimation {
        id: entryAnim
        PauseAnimation { duration: noteIndex * 30 }
        ParallelAnimation {
            NumberAnimation {
                target: noteCard
                property: "opacity"
                from: 0
                to: 1
                duration: 200
                easing.type: Easing.OutCubic
            }
            NumberAnimation {
                target: noteCard
                property: "y"
                from: noteCard.y + 8
                to: noteCard.y
                duration: 200
                easing.type: Easing.OutCubic
            }
        }
    }

    Timer {
        id: saveTimer
        interval: 500
        repeat: false
        onTriggered: {
            if (noteModel && dirty) {
                noteModel.update_content(noteIndex, editField.text);
                dirty = false;
                editing = false;
            }
        }
    }

    // Read mode
    ColumnLayout {
        id: readContent
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingXs
        visible: !editing

        MouseArea {
            Layout.fillWidth: true
            Layout.fillHeight: true
            onClicked: {
                editing = true;
                editField.text = noteModel.get_content(noteIndex);
                editField.forceActiveFocus();
            }
            cursorShape: Qt.IBeamCursor

            ColumnLayout {
                width: parent.width
                spacing: Theme.spacingXs

                Repeater {
                    model: noteModel
                        ? (noteModel.get_is_checklist(noteIndex)
                            ? parseChecklistContent(noteModel.get_content(noteIndex))
                            : [{ text: noteModel.get_content(noteIndex), checked: false }])
                        : []

                    delegate: RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingSm

                        CheckBox {
                            visible: noteModel && noteModel.get_is_checklist(noteIndex)
                            checked: modelData.checked
                            onClicked: {
                                const content = noteModel.get_content(noteIndex);
                                const lines = content.split("\n");
                                const idx = index;
                                if (idx >= 0 && idx < lines.length) {
                                    const line = lines[idx];
                                    const newPrefix = modelData.checked ? "- [ ] " : "- [x] ";
                                    const item = line.replace(/^-\s*\[[ x]\]\s*/i, "").trim();
                                    lines[idx] = newPrefix + item;
                                    noteModel.update_content(noteIndex, lines.join("\n"));
                                }
                            }
                        }

                        Label {
                            text: modelData.text
                            font.pixelSize: Theme.fontSizeNormal
                            font.strikeout: modelData.checked
                            color: Theme.text
                            opacity: modelData.checked ? 0.6 : 1.0
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                Label {
                    text: noteModel ? noteModel.get_created_at(noteIndex) : ""
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.textMuted
                    visible: text.length > 0
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingXs
                    visible: !editing

                    ToolButton {
                        text: Icons.squaresFour
                        font.family: Icons.family
                        font.pixelSize: 14
                        onClicked: noteCard.promoteRequested(noteIndex, noteModel.get_content(noteIndex))
                        ToolTip.text: "Promote to Project"
                        ToolTip.visible: hovered

                        background: Rectangle {
                            radius: Theme.buttonRadius
                            color: parent.hovered ? Theme.primary + "30" : "transparent"
                        }

                        contentItem: Text {
                            text: parent.text
                            font.family: Icons.family
                            color: Theme.primary
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

                    ToolButton {
                        text: noteModel.get_done(noteIndex) ? "Undo" : "Done"
                        font.pixelSize: Theme.fontSizeSmall
                        onClicked: noteModel.toggle_done(noteIndex)

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
                        text: noteModel.get_archived(noteIndex) ? "Restore" : Icons.trash
                        font.family: Icons.family
                        font.pixelSize: 14
                        onClicked: noteModel.get_archived(noteIndex)
                            ? noteModel.unarchive_note(noteIndex)
                            : noteModel.archive_note(noteIndex)
                        ToolTip.text: noteModel.get_archived(noteIndex) ? "Restore" : "Archive"
                        ToolTip.visible: hovered

                        background: Rectangle {
                            radius: Theme.buttonRadius
                            color: parent.hovered ? Theme.error + "30" : "transparent"
                        }

                        contentItem: Text {
                            text: parent.text
                            font.family: Icons.family
                            color: Theme.error
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }
            }
        }
    }

    signal promoteRequested(int index, string title)

    // Edit mode
    ColumnLayout {
        id: editContent
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        visible: editing

        TextArea {
            id: editField
            Layout.fillWidth: true
            Layout.preferredHeight: 100
            wrapMode: TextEdit.Wrap
            color: Theme.text
            font.pixelSize: Theme.fontSizeNormal
            padding: 4

            background: Rectangle {
                color: "transparent"
            }

            onTextChanged: {
                dirty = true;
                saveTimer.restart();
            }

            onActiveFocusChanged: {
                if (!activeFocus && dirty) {
                    saveTimer.stop();
                    if (noteModel) {
                        noteModel.update_content(noteIndex, text);
                    }
                    dirty = false;
                    editing = false;
                }
            }
        }
    }

    function parseChecklistContent(content) {
        const lines = content.split("\n");
        return lines.map(line => {
            const uncheckedMatch = line.match(/^-\s*\[\s*\]\s*(.*)$/i);
            const checkedMatch = line.match(/^-\s*\[[x]\]\s*(.*)$/i);
            if (uncheckedMatch) {
                return { text: uncheckedMatch[1].trim(), checked: false };
            }
            if (checkedMatch) {
                return { text: checkedMatch[1].trim(), checked: true };
            }
            return { text: line.trim(), checked: false };
        });
    }
}
