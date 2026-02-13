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

                Flow {
                    Layout.fillWidth: true
                    spacing: Theme.spacingXs
                    visible: noteModel && noteModel.get_labels(noteIndex).length > 0

                    Repeater {
                        model: noteModel ? noteModel.get_labels(noteIndex) : []

                        delegate: Rectangle {
                            width: labelText.implicitWidth + Theme.spacingSm * 2
                            height: labelText.implicitHeight + Theme.spacingXs
                            radius: 4
                            color: Theme.primary + "20"

                            Label {
                                id: labelText
                                anchors.centerIn: parent
                                text: modelData
                                font.pixelSize: Theme.fontSizeSmall
                                color: Theme.primary
                            }

                            MouseArea {
                                anchors.fill: parent
                                onClicked: noteModel.remove_label(noteIndex, modelData)
                            }
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
                    visible: noteModel && noteModel.get_reminder(noteIndex).length > 0
                    spacing: Theme.spacingXs
                    Label {
                        text: Icons.clock
                        font.family: Icons.family
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.primary
                    }
                    Label {
                        text: noteModel ? noteModel.get_reminder(noteIndex) : ""
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.primary
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingXs
                    visible: !editing

                    ToolButton {
                        text: noteModel.get_pinned(noteIndex) ? Icons.starFill : Icons.pushPin
                        font.family: Icons.family
                        font.pixelSize: 14
                        onClicked: noteModel.set_pinned(noteIndex, !noteModel.get_pinned(noteIndex))
                        ToolTip.text: noteModel.get_pinned(noteIndex) ? "Unpin" : "Pin"
                        ToolTip.visible: hovered

                        background: Rectangle {
                            radius: Theme.buttonRadius
                            color: parent.hovered ? Theme.primary + "30" : "transparent"
                        }

                        contentItem: Text {
                            text: parent.text
                            font.family: Icons.family
                            color: noteModel.get_pinned(noteIndex) ? Theme.primary : Theme.textMuted
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

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
                        text: noteModel.get_archived(noteIndex) ? "Restore" : Icons.archiveBox
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
                            color: Theme.textMuted
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

                    ToolButton {
                        id: overflowBtn
                        text: Icons.dotsThree
                        font.family: Icons.family
                        font.pixelSize: 14
                        onClicked: overflowMenu.open()

                        background: Rectangle {
                            radius: Theme.buttonRadius
                            color: parent.hovered ? Theme.surfaceHover : "transparent"
                        }

                        contentItem: Text {
                            text: parent.text
                            font.family: Icons.family
                            color: Theme.textMuted
                            font.pixelSize: 14
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }

                        Menu {
                            id: overflowMenu
                            width: 180

                            MenuItem {
                                text: "Change color"
                                onTriggered: colorPicker.open()
                            }
                            MenuItem {
                                text: noteModel.get_pinned(noteIndex) ? "Unpin" : "Pin"
                                onTriggered: noteModel.set_pinned(noteIndex, !noteModel.get_pinned(noteIndex))
                            }
                            MenuItem {
                                text: noteModel.get_archived(noteIndex) ? "Restore" : "Archive"
                                onTriggered: noteModel.get_archived(noteIndex)
                                    ? noteModel.unarchive_note(noteIndex)
                                    : noteModel.archive_note(noteIndex)
                            }
                            MenuItem {
                                text: "Promote to Project"
                                onTriggered: noteCard.promoteRequested(noteIndex, noteModel.get_content(noteIndex))
                            }
                            MenuItem {
                                text: "Add label"
                                onTriggered: addLabelPopup.open()
                            }
                            MenuItem {
                                text: noteModel.get_reminder(noteIndex).length > 0 ? "Remove reminder" : "Add reminder"
                                onTriggered: {
                                    if (noteModel.get_reminder(noteIndex).length > 0) {
                                        noteModel.set_reminder(noteIndex, "");
                                    } else {
                                        reminderPopup.open();
                                    }
                                }
                            }
                        }
                    }

                    Popup {
                        id: reminderPopup
                        width: 280
                        padding: Theme.spacingMd

                        background: Rectangle {
                            color: Theme.surface
                            border.color: Theme.border
                            border.width: 1
                            radius: Theme.cardRadius
                        }

                        onOpened: {
                            const now = new Date();
                            reminderDateField.text = now.getFullYear() + "-"
                                + String(now.getMonth() + 1).padStart(2, "0") + "-"
                                + String(now.getDate()).padStart(2, "0");
                            reminderHour.value = now.getHours();
                            reminderMinute.value = now.getMinutes();
                        }

                        ColumnLayout {
                            width: parent.width - Theme.spacingMd * 2
                            spacing: Theme.spacingSm

                            Label {
                                text: "Set reminder"
                                font.pixelSize: Theme.fontSizeMedium
                                font.bold: true
                                color: Theme.text
                            }
                            TextField {
                                id: reminderDateField
                                placeholderText: "YYYY-MM-DD"
                                Layout.fillWidth: true
                                font.pixelSize: Theme.fontSizeNormal

                                background: Rectangle {
                                    color: Theme.inputBg
                                    border.color: Theme.inputBorder
                                    border.width: 1
                                    radius: Theme.inputRadius
                                }
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: Theme.spacingSm
                                Label {
                                    text: "Time:"
                                    font.pixelSize: Theme.fontSizeSmall
                                    color: Theme.textSecondary
                                }
                                SpinBox {
                                    id: reminderHour
                                    from: 0
                                    to: 23
                                    value: 9
                                }
                                Label { text: ":"; color: Theme.text }
                                SpinBox {
                                    id: reminderMinute
                                    from: 0
                                    to: 59
                                    value: 0
                                }
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                Button {
                                    text: "Set"
                                    onClicked: {
                                        const parts = reminderDateField.text.trim().split(/[-T\s]/);
                                        if (parts.length >= 3) {
                                            const year = parseInt(parts[0], 10);
                                            const month = parseInt(parts[1], 10);
                                            const day = parseInt(parts[2], 10);
                                            if (!isNaN(year) && !isNaN(month) && !isNaN(day)
                                                    && month >= 1 && month <= 12 && day >= 1 && day <= 31) {
                                                const iso = String(year) + "-"
                                                    + String(month).padStart(2, "0") + "-"
                                                    + String(day).padStart(2, "0") + "T"
                                                    + String(reminderHour.value).padStart(2, "0") + ":"
                                                    + String(reminderMinute.value).padStart(2, "0") + ":00Z";
                                                noteModel.set_reminder(noteIndex, iso);
                                                reminderPopup.close();
                                            }
                                        }
                                    }
                                }
                                Button {
                                    text: "Cancel"
                                    flat: true
                                    onClicked: reminderPopup.close()
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ColorPicker {
        id: colorPicker
        onColorPicked: (hex) => noteModel.set_color(noteIndex, hex)
    }

    Popup {
        id: addLabelPopup
        width: 220
        padding: Theme.spacingMd

        background: Rectangle {
            color: Theme.surface
            border.color: Theme.border
            border.width: 1
            radius: Theme.cardRadius
        }

        ColumnLayout {
            width: parent.width - Theme.spacingMd * 2
            spacing: Theme.spacingSm

            Label {
                text: "Add label"
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
                color: Theme.text
            }
            TextField {
                id: labelField
                placeholderText: "Label name"
                Layout.fillWidth: true
                onAccepted: {
                    if (text.trim().length > 0) {
                        noteModel.add_label(noteIndex, text.trim());
                    }
                    text = "";
                    addLabelPopup.close();
                }
            }
            Button {
                text: "Add"
                onClicked: {
                    if (labelField.text.trim().length > 0) {
                        noteModel.add_label(noteIndex, labelField.text.trim());
                    }
                    labelField.text = "";
                    addLabelPopup.close();
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
