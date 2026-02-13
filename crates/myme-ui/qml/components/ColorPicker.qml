import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import ".."

Popup {
    id: colorPicker
    width: 200
    height: colorGrid.implicitHeight + Theme.spacingMd * 2
    padding: Theme.spacingMd

    property string selectedColor: ""
    signal colorPicked(string hex)

    readonly property var keepColors: [
        "#ffffff", "#fef9c3", "#fecaca", "#fecdd3", "#e9d5ff",
        "#ddd6fe", "#bfdbfe", "#a5f3fc", "#ccfbf1", "#d1d5db"
    ]

    background: Rectangle {
        color: Theme.surface
        border.color: Theme.border
        border.width: 1
        radius: Theme.cardRadius
    }

    GridLayout {
        id: colorGrid
        columns: 5
        rowSpacing: Theme.spacingSm
        columnSpacing: Theme.spacingSm

        Repeater {
            model: keepColors

            delegate: Rectangle {
                required property string modelData
                Layout.preferredWidth: 32
                Layout.preferredHeight: 32
                radius: 4
                color: modelData
                border.width: selectedColor === modelData ? 2 : 0
                border.color: Theme.primary

                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        selectedColor = modelData;
                        colorPicked(modelData);
                        colorPicker.close();
                    }
                }
            }
        }

        Rectangle {
            Layout.preferredWidth: 32
            Layout.preferredHeight: 32
            radius: 4
            color: "transparent"
            border.width: 1
            border.color: Theme.border

            Label {
                anchors.centerIn: parent
                text: Icons.x
                font.family: Icons.family
                font.pixelSize: 14
                color: Theme.textMuted
            }

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    selectedColor = "";
                    colorPicked("");
                    colorPicker.close();
                }
            }
        }
    }
}
