import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: detailPage
    title: organizationName

    required property string organizationId
    required property string organizationName

    property int currentTab: 0

    onCurrentTabChanged: {
        importResult = ""
        importHadError = false
    }

    // NOTE: import_rfp_leads is synchronous on the Qt main thread, so importingLeads
    // will not visually update before the call returns. Reserved for a future async refactor.
    property bool importingLeads: false
    property bool importHadError: false
    property string importResult: ""

    background: Rectangle {
        color: Theme.background
    }

    ProspectModel {
        id: prospectModel
    }

    Timer {
        id: prospectPollTimer
        interval: 100
        running: prospectModel.loading
        repeat: true
        onTriggered: prospectModel.poll_channel()
    }

    Connections {
        target: prospectModel
        function onProspects_changed() {
            prospectModel.prospect_count(); // trigger UI refresh
        }
    }

    // Pipeline stage definitions
    readonly property var stages: [
        { key: "lead", label: "Lead", color: "#8a8580" },
        { key: "qualified", label: "Qualified", color: "#64b5f6" },
        { key: "contacted", label: "Contacted", color: "#e5a54b" },
        { key: "proposal", label: "Proposal", color: "#b39ddb" },
        { key: "negotiation", label: "Negotiation", color: "#ff8a65" },
        { key: "won", label: "Won", color: "#5bb98c" },
        { key: "lost", label: "Lost", color: "#e57373" }
    ]

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 64
            color: "transparent"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacingLg
                anchors.rightMargin: Theme.spacingLg
                spacing: Theme.spacingMd

                // Back button
                Button {
                    contentItem: RowLayout {
                        spacing: Theme.spacingXs
                        Text {
                            text: Icons.caretLeft
                            font.family: Icons.family
                            font.pixelSize: 18
                            color: Theme.textSecondary
                        }
                        Label {
                            text: "Organizations"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                        }
                    }
                    background: Rectangle {
                        color: parent.hovered ? (Theme.isDark ? "#ffffff10" : "#00000008") : "transparent"
                        radius: Theme.buttonRadius
                    }
                    onClicked: AppContext.pageStack.pop()
                }

                Text {
                    text: Icons.buildings
                    font.family: Icons.family
                    font.pixelSize: 24
                    color: Theme.primary
                }

                Label {
                    text: detailPage.organizationName
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeLarge
                    font.weight: Font.Bold
                    color: Theme.text
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }
            }
        }

        // Tab bar
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: "transparent"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.spacingLg
                spacing: Theme.spacingMd

                Repeater {
                    model: ["Pipeline", "Projects", "Notes"]

                    Rectangle {
                        Layout.preferredHeight: 36
                        Layout.preferredWidth: tabLabel.implicitWidth + Theme.spacingLg * 2
                        radius: Theme.buttonRadius
                        color: detailPage.currentTab === index ? (Theme.isDark ? "#ffffff15" : "#00000008") : (tabMouse.containsMouse ? (Theme.isDark ? "#ffffff08" : "#00000004") : "transparent")

                        Label {
                            id: tabLabel
                            anchors.centerIn: parent
                            text: modelData
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            font.weight: detailPage.currentTab === index ? Font.Bold : Font.Normal
                            color: detailPage.currentTab === index ? Theme.primary : Theme.textSecondary
                        }

                        // Active indicator
                        Rectangle {
                            visible: detailPage.currentTab === index
                            anchors.bottom: parent.bottom
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: parent.width - Theme.spacingMd
                            height: 2
                            radius: 1
                            color: Theme.primary
                        }

                        MouseArea {
                            id: tabMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: detailPage.currentTab = index
                        }
                    }
                }

                Item { Layout.fillWidth: true }
            }
        }

        // Separator
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.borderLight
        }

        // Error banner
        Rectangle {
            visible: prospectModel.error_message !== ""
            Layout.fillWidth: true
            Layout.margins: Theme.spacingMd
            height: visible ? detailErrorLabel.implicitHeight + Theme.spacingMd * 2 : 0
            color: Theme.isDark ? "#3d2020" : "#fde8e8"
            radius: Theme.cardRadius
            border.color: "transparent"

            Label {
                id: detailErrorLabel
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                text: prospectModel.error_message
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.isDark ? "#f5a5a5" : "#c53030"
                wrapMode: Text.WordWrap
            }
        }

        // Tab content
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: detailPage.currentTab

            // Tab 0: Pipeline
            Item {
                id: pipelineTab

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    // Find Leads toolbar
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingMd
                        Layout.leftMargin: Theme.spacingMd
                        Layout.rightMargin: Theme.spacingMd
                        spacing: Theme.spacingMd

                        Item { Layout.fillWidth: true }

                        Label {
                            visible: importResult !== ""
                            text: importResult
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            color: importHadError ? (Theme.isDark ? "#f5a5a5" : "#c53030") : Theme.textSecondary
                        }

                        Button {
                            text: importingLeads ? "Importing..." : "Find Leads"
                            enabled: !importingLeads
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            contentItem: Label {
                                text: parent.text
                                font: parent.font
                                color: Theme.primaryText
                                horizontalAlignment: Text.AlignHCenter
                            }
                            background: Rectangle {
                                color: parent.enabled ? (parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary) : (Theme.isDark ? "#ffffff30" : "#00000020")
                                radius: Theme.buttonRadius
                                implicitHeight: 32
                                implicitWidth: 110
                            }
                            onClicked: {
                                importingLeads = true
                                importHadError = false
                                importResult = ""
                                var resultJson = prospectModel.import_rfp_leads(detailPage.organizationId)
                                importingLeads = false
                                try {
                                    var result = JSON.parse(resultJson)
                                    if (result.error) {
                                        importHadError = true
                                        importResult = "Error: " + result.error
                                    } else if (result.failed > 0) {
                                        importHadError = true
                                        importResult = result.imported + " imported, " + result.failed + " failed, " + result.skipped + " skipped"
                                    } else {
                                        importHadError = false
                                        importResult = result.imported + " leads imported (" + result.skipped + " skipped)"
                                    }
                                } catch (e) {
                                    console.error("Find Leads JSON parse failed:", e, "raw:", resultJson)
                                    importHadError = true
                                    importResult = "Error processing results: " + resultJson.substring(0, 100)
                                }
                            }
                        }
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.leftMargin: Theme.spacingMd
                        Layout.rightMargin: Theme.spacingMd
                        Layout.bottomMargin: Theme.spacingMd
                        contentWidth: pipelineRow.implicitWidth

                        RowLayout {
                            id: pipelineRow
                            spacing: Theme.spacingSm
                            height: parent.height

                            Repeater {
                                model: detailPage.stages

                                // Stage column
                                Rectangle {
                                    Layout.preferredWidth: 220
                                    Layout.fillHeight: true
                                    color: Theme.isDark ? "#ffffff05" : "#00000003"
                                    radius: Theme.cardRadius
                                    border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                                    border.width: 1

                                    ColumnLayout {
                                        anchors.fill: parent
                                        anchors.margins: Theme.spacingSm
                                        spacing: Theme.spacingSm

                                        // Stage header
                                        RowLayout {
                                            Layout.fillWidth: true
                                            spacing: Theme.spacingSm

                                            Rectangle {
                                                width: 10
                                                height: 10
                                                radius: 5
                                                color: modelData.color
                                            }

                                            Label {
                                                text: modelData.label
                                                font.family: Theme.fontFamily
                                                font.pixelSize: Theme.fontSizeSmall
                                                font.weight: Font.Bold
                                                color: Theme.text
                                                Layout.fillWidth: true
                                            }

                                            Label {
                                                text: prospectModel.count_for_stage(modelData.key)
                                                font.family: Theme.fontFamily
                                                font.pixelSize: Theme.fontSizeSmall
                                                color: Theme.textSecondary
                                            }
                                        }

                                        // Prospect cards
                                        ListView {
                                            Layout.fillWidth: true
                                            Layout.fillHeight: true
                                            clip: true
                                            spacing: Theme.spacingXs

                                            model: {
                                                const indices = JSON.parse(prospectModel.prospects_for_stage(modelData.key));
                                                return indices;
                                            }

                                            delegate: Rectangle {
                                                width: ListView.view.width
                                                height: prospectCardLayout.implicitHeight + Theme.spacingSm * 2
                                                radius: Theme.buttonRadius
                                                color: prospectMouse.containsMouse ? (Theme.isDark ? Qt.lighter(Theme.cardBg, 1.05) : Qt.darker(Theme.cardBg, 1.02)) : Theme.cardBg
                                                border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                                                border.width: 1

                                                property int prospectIndex: modelData

                                                // Staggered animation
                                                opacity: 0
                                                Component.onCompleted: prospectFade.start()
                                                SequentialAnimation {
                                                    id: prospectFade
                                                    PauseAnimation { duration: index * 30 }
                                                    NumberAnimation { target: parent; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                                                }

                                                ColumnLayout {
                                                    id: prospectCardLayout
                                                    anchors.left: parent.left
                                                    anchors.right: parent.right
                                                    anchors.top: parent.top
                                                    anchors.margins: Theme.spacingSm
                                                    spacing: 2

                                                    Label {
                                                        text: prospectModel.get_prospect_name(prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.weight: Font.Medium
                                                        color: Theme.text
                                                        elide: Text.ElideRight
                                                        Layout.fillWidth: true
                                                    }

                                                    Label {
                                                        visible: text !== ""
                                                        text: prospectModel.get_prospect_value(prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: 11
                                                        color: Theme.primary
                                                    }

                                                    Label {
                                                        visible: text !== ""
                                                        text: prospectModel.get_prospect_contact_name(prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: 11
                                                        color: Theme.textSecondary
                                                        elide: Text.ElideRight
                                                        Layout.fillWidth: true
                                                    }
                                                }

                                                MouseArea {
                                                    id: prospectMouse
                                                    anchors.fill: parent
                                                    hoverEnabled: true
                                                    cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        editProspectIndex = prospectIndex;
                                                        editProspectDialog.open();
                                                    }
                                                }
                                            }
                                        }

                                        // Add prospect button
                                        Button {
                                            Layout.fillWidth: true
                                            contentItem: Label {
                                                text: "+ Add"
                                                font.family: Theme.fontFamily
                                                font.pixelSize: Theme.fontSizeSmall
                                                color: Theme.textSecondary
                                                horizontalAlignment: Text.AlignHCenter
                                            }
                                            background: Rectangle {
                                                color: parent.hovered ? (Theme.isDark ? "#ffffff08" : "#00000005") : "transparent"
                                                radius: Theme.buttonRadius
                                                border.color: Theme.isDark ? "#ffffff10" : "#00000010"
                                                border.width: 1
                                                implicitHeight: 32
                                            }
                                            onClicked: {
                                                addProspectStage = modelData.key;
                                                addProspectDialog.open();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }  // closes ColumnLayout
            }

            // Tab 1: Projects
            Item {
                id: projectsTab

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLg
                    spacing: Theme.spacingMd

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingMd

                        Label {
                            text: "Linked Projects"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            font.weight: Font.Bold
                            color: Theme.text
                            Layout.fillWidth: true
                        }

                        Button {
                            text: "+ Link Project"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            contentItem: Label {
                                text: parent.text
                                font: parent.font
                                color: Theme.primaryText
                                horizontalAlignment: Text.AlignHCenter
                            }
                            background: Rectangle {
                                color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary
                                radius: Theme.buttonRadius
                                implicitHeight: 32
                                implicitWidth: 120
                            }
                            onClicked: linkProjectDialog.open()
                        }
                    }

                    ListView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        spacing: Theme.spacingSm

                        model: {
                            const projects = JSON.parse(prospectModel.linked_projects());
                            return projects;
                        }

                        delegate: Rectangle {
                            width: ListView.view.width
                            height: 48
                            radius: Theme.buttonRadius
                            color: projectCardMouse.containsMouse ? (Theme.isDark ? Qt.lighter(Theme.cardBg, 1.05) : Qt.darker(Theme.cardBg, 1.02)) : Theme.cardBg
                            border.color: Theme.isDark ? "#ffffff08" : "#00000008"
                            border.width: 1

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.spacingSm
                                spacing: Theme.spacingSm

                                Text {
                                    text: Icons.squaresFour
                                    font.family: Icons.family
                                    font.pixelSize: 16
                                    color: Theme.primary
                                }

                                Label {
                                    text: modelData
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeNormal
                                    color: Theme.text
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                }

                                Button {
                                    contentItem: Text {
                                        text: Icons.x
                                        font.family: Icons.family
                                        font.pixelSize: 14
                                        color: Theme.textSecondary
                                        horizontalAlignment: Text.AlignHCenter
                                    }
                                    background: Rectangle {
                                        color: parent.hovered ? (Theme.isDark ? "#ffffff10" : "#00000008") : "transparent"
                                        radius: Theme.buttonRadius
                                        implicitWidth: 28
                                        implicitHeight: 28
                                    }
                                    onClicked: prospectModel.unlink_project(modelData)
                                }
                            }

                            MouseArea {
                                id: projectCardMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.NoButton
                            }
                        }

                        // Empty state
                        Label {
                            visible: parent.count === 0
                            anchors.centerIn: parent
                            text: "No linked projects"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.textSecondary
                        }
                    }
                }
            }

            // Tab 2: Notes
            Item {
                id: notesTab

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLg
                    spacing: Theme.spacingSm

                    Label {
                        text: "Organization Notes"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeNormal
                        font.weight: Font.Bold
                        color: Theme.text
                    }

                    Label {
                        text: "Notes persistence coming soon. Notes typed here will not be saved."
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.primary
                        opacity: 0.8
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        TextArea {
                            id: notesArea
                            text: ""
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            wrapMode: TextArea.Wrap
                            placeholderText: "Write notes about this organization..."

                            background: Rectangle {
                                color: Theme.isDark ? "#ffffff05" : "#00000003"
                                radius: Theme.cardRadius
                                border.color: notesArea.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff10" : "#00000010")
                                border.width: 1
                            }
                        }
                    }
                }
            }
        }
    }

    // State for add prospect dialog
    property string addProspectStage: "lead"
    property int editProspectIndex: -1

    // Add Prospect Dialog
    Dialog {
        id: addProspectDialog
        title: "Add Prospect"
        modal: true
        anchors.centerIn: parent
        width: Math.min(450, parent.width - 40)

        background: Rectangle {
            color: Theme.cardBg
            radius: Theme.cardRadius
            border.color: Theme.isDark ? "#ffffff15" : "#00000015"
            border.width: 1
        }

        header: Label {
            text: "Add Prospect"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeLarge
            font.weight: Font.Bold
            color: Theme.text
            padding: Theme.spacingLg
        }

        contentItem: ColumnLayout {
            spacing: Theme.spacingSm

            Label { text: "Name *"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: prospectNameField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "Prospect name"
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: prospectNameField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Value"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: prospectValueField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "$10,000"
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: prospectValueField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Contact Name"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: prospectContactField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "Contact name"
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: prospectContactField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Description"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextArea {
                id: prospectDescField
                Layout.fillWidth: true
                Layout.preferredHeight: 60
                font.family: Theme.fontFamily
                placeholderText: "Description"
                color: Theme.text
                wrapMode: TextArea.Wrap
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: prospectDescField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }
        }

        footer: DialogButtonBox {
            background: Rectangle { color: "transparent" }

            Button {
                text: "Cancel"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.textSecondary; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#ffffff10" : "#00000008") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Create"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
            }
        }

        onAccepted: {
            prospectModel.create_prospect(
                detailPage.organizationId,
                prospectNameField.text,
                prospectDescField.text,
                addProspectStage,
                prospectValueField.text,
                prospectContactField.text,
                "",
                ""
            );
            prospectNameField.text = "";
            prospectValueField.text = "";
            prospectContactField.text = "";
            prospectDescField.text = "";
        }

        onRejected: {
            prospectNameField.text = "";
            prospectValueField.text = "";
            prospectContactField.text = "";
            prospectDescField.text = "";
        }
    }

    // Edit Prospect Dialog
    Dialog {
        id: editProspectDialog
        title: "Edit Prospect"
        modal: true
        anchors.centerIn: parent
        width: Math.min(450, parent.width - 40)

        background: Rectangle {
            color: Theme.cardBg
            radius: Theme.cardRadius
            border.color: Theme.isDark ? "#ffffff15" : "#00000015"
            border.width: 1
        }

        header: RowLayout {
            spacing: Theme.spacingMd
            Label {
                text: "Edit Prospect"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Bold
                color: Theme.text
                Layout.leftMargin: Theme.spacingLg
                Layout.topMargin: Theme.spacingLg
                Layout.fillWidth: true
            }
            Button {
                text: "Delete"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                Layout.rightMargin: Theme.spacingLg
                Layout.topMargin: Theme.spacingLg
                contentItem: Label { text: parent.text; font: parent.font; color: "#e57373"; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? "#e5737320" : "transparent"; radius: Theme.buttonRadius; implicitHeight: 32 }
                onClicked: {
                    prospectModel.delete_prospect(editProspectIndex);
                    editProspectDialog.close();
                }
            }
        }

        contentItem: ColumnLayout {
            spacing: Theme.spacingSm

            Label { text: "Name *"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: editNameField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: editNameField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Value"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: editValueField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: editValueField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Contact"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: editContactField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: editContactField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }

            Label { text: "Description"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextArea {
                id: editDescField
                Layout.fillWidth: true
                Layout.preferredHeight: 60
                font.family: Theme.fontFamily
                color: Theme.text
                wrapMode: TextArea.Wrap
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: editDescField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }
        }

        footer: DialogButtonBox {
            background: Rectangle { color: "transparent" }

            Button {
                text: "Cancel"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.textSecondary; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#ffffff10" : "#00000008") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Save"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
            }
        }

        onAboutToShow: {
            if (editProspectIndex >= 0) {
                editNameField.text = prospectModel.get_prospect_name(editProspectIndex);
                editValueField.text = prospectModel.get_prospect_value(editProspectIndex);
                editContactField.text = prospectModel.get_prospect_contact_name(editProspectIndex);
                editDescField.text = prospectModel.get_prospect_description(editProspectIndex);
            }
        }

        onAccepted: {
            if (editProspectIndex >= 0) {
                prospectModel.update_prospect(
                    editProspectIndex,
                    editNameField.text,
                    editDescField.text,
                    editValueField.text,
                    editContactField.text,
                    "",
                    ""
                );
            }
        }
    }

    // Link Project Dialog
    Dialog {
        id: linkProjectDialog
        title: "Link Project"
        modal: true
        anchors.centerIn: parent
        width: Math.min(400, parent.width - 40)

        background: Rectangle {
            color: Theme.cardBg
            radius: Theme.cardRadius
            border.color: Theme.isDark ? "#ffffff15" : "#00000015"
            border.width: 1
        }

        header: Label {
            text: "Link Project"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeLarge
            font.weight: Font.Bold
            color: Theme.text
            padding: Theme.spacingLg
        }

        contentItem: ColumnLayout {
            spacing: Theme.spacingSm

            Label { text: "Project ID"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: linkProjectField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "Enter project ID"
                color: Theme.text
                background: Rectangle {
                    color: Theme.isDark ? "#ffffff08" : "#00000005"
                    radius: Theme.buttonRadius
                    border.color: linkProjectField.activeFocus ? Theme.primary : (Theme.isDark ? "#ffffff15" : "#00000015")
                    border.width: 1
                }
            }
        }

        footer: DialogButtonBox {
            background: Rectangle { color: "transparent" }

            Button {
                text: "Cancel"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.textSecondary; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#ffffff10" : "#00000008") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Link"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
            }
        }

        onAccepted: {
            if (linkProjectField.text.trim()) {
                prospectModel.link_project(linkProjectField.text.trim());
            }
            linkProjectField.text = "";
        }

        onRejected: {
            linkProjectField.text = "";
        }
    }

    Component.onCompleted: {
        prospectModel.load_prospects(detailPage.organizationId);
    }
}
