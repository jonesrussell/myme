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
        prospectModel.import_result = ""
    }

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

    // prospect_revision is a QProperty counter that increments on every change,
    // allowing QML bindings that read it to re-evaluate automatically.

    // Pipeline stage definitions
    readonly property var stages: [
        { key: "lead", label: "Lead", color: "#8A8580" },
        { key: "qualified", label: "Qualified", color: "#64B5F6" },
        { key: "contacted", label: "Contacted", color: "#F59E0B" },
        { key: "proposal", label: "Proposal", color: "#B39DDB" },
        { key: "negotiation", label: "Negotiation", color: "#FF8A65" },
        { key: "won", label: "Won", color: "#5BB98C" },
        { key: "lost", label: "Lost", color: "#E57373" }
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
                        color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"
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
                        color: detailPage.currentTab === index ? (Theme.isDark ? "#15ffffff" : "#08000000") : (tabMouse.containsMouse ? (Theme.isDark ? "#08ffffff" : "#04000000") : "transparent")

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
            color: Theme.errorBg
            radius: Theme.cardRadius
            border.color: "transparent"

            Label {
                id: detailErrorLabel
                anchors.fill: parent
                anchors.margins: Theme.spacingMd
                text: prospectModel.error_message
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.error
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
                            visible: importStatusText !== ""
                            text: importStatusText
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            color: importStatusIsError ? Theme.error : Theme.textSecondary

                            property string importStatusText: {
                                var raw = prospectModel.import_result
                                if (raw === "" || raw === '{"pending":true}') return ""
                                try {
                                    var r = JSON.parse(raw)
                                    if (r.error) return "Error: " + r.error
                                    if (r.failed > 0) return r.imported + " imported, " + r.failed + " failed"
                                    return r.imported + " leads imported (" + r.skipped + " skipped)"
                                } catch (e) { return "Error processing results" }
                            }
                            property bool importStatusIsError: {
                                var raw = prospectModel.import_result
                                if (raw === "") return false
                                try {
                                    var r = JSON.parse(raw)
                                    return !!r.error || (r.failed > 0)
                                } catch (e) { return true }
                            }
                        }

                        Button {
                            text: prospectModel.loading ? "Importing..." : "Find Leads"
                            enabled: !prospectModel.loading
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            contentItem: Label {
                                text: parent.text
                                font: parent.font
                                color: Theme.primaryText
                                horizontalAlignment: Text.AlignHCenter
                            }
                            background: Rectangle {
                                color: parent.enabled ? (parent.hovered ? Theme.primaryHover : Theme.primary) : (Theme.isDark ? "#30ffffff" : "#20000000")
                                radius: Theme.buttonRadius
                                implicitHeight: 32
                                implicitWidth: 110
                            }
                            onClicked: {
                                prospectModel.import_rfp_leads(detailPage.organizationId)
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
                                    color: Theme.isDark ? "#05ffffff" : "#03000000"
                                    radius: Theme.cardRadius
                                    border.color: Theme.cardBorderSubtle
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
                                                text: { void(prospectModel.prospect_revision); return prospectModel.count_for_stage(modelData.key) }
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
                                                void(prospectModel.prospect_revision);
                                                const indices = JSON.parse(prospectModel.prospects_for_stage(modelData.key));
                                                return indices;
                                            }

                                            delegate: Rectangle {
                                                id: prospectCard
                                                width: ListView.view.width
                                                height: prospectCardLayout.implicitHeight + Theme.spacingSm * 2
                                                radius: Theme.buttonRadius
                                                color: prospectMouse.containsMouse ? (Theme.isDark ? Qt.lighter(Theme.cardBg, 1.05) : Qt.darker(Theme.cardBg, 1.02)) : Theme.cardBg
                                                border.color: Theme.cardBorderSubtle
                                                border.width: 1

                                                property int prospectIndex: modelData

                                                // Staggered animation
                                                opacity: 0
                                                Component.onCompleted: {
                                                    console.log("[ProspectCard] index=" + index
                                                        + " prospectIndex=" + prospectIndex
                                                        + " name=" + prospectModel.get_prospect_name(prospectIndex)
                                                        + " height=" + height
                                                        + " implicitH=" + prospectCardLayout.implicitHeight)
                                                    prospectFade.start()
                                                }
                                                SequentialAnimation {
                                                    id: prospectFade
                                                    PauseAnimation { duration: index * 30 }
                                                    NumberAnimation { target: prospectCard; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                                                }

                                                ColumnLayout {
                                                    id: prospectCardLayout
                                                    anchors.left: parent.left
                                                    anchors.right: parent.right
                                                    anchors.top: parent.top
                                                    anchors.margins: Theme.spacingSm
                                                    spacing: Theme.spacingXxs

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
                                                color: parent.hovered ? Theme.surfaceAlt : "transparent"
                                                radius: Theme.buttonRadius
                                                border.color: Theme.isDark ? "#10ffffff" : "#10000000"
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
                                color: parent.hovered ? Theme.primaryHover : Theme.primary
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
                            border.color: Theme.cardBorderSubtle
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
                                        color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"
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
                                color: Theme.isDark ? "#05ffffff" : "#03000000"
                                radius: Theme.cardRadius
                                border.color: notesArea.activeFocus ? Theme.primary : (Theme.isDark ? "#10ffffff" : "#10000000")
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
            border.color: Theme.borderLight
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: prospectNameField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: prospectValueField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: prospectContactField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: prospectDescField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Create"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Theme.primaryHover : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
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
            border.color: Theme.borderLight
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
                contentItem: Label { text: parent.text; font: parent.font; color: "#E57373"; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? "#20E57373" : "transparent"; radius: Theme.buttonRadius; implicitHeight: 32 }
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: editNameField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: editValueField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: editContactField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: editDescField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Save"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Theme.primaryHover : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
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
            border.color: Theme.borderLight
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
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: linkProjectField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                background: Rectangle { color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"; radius: Theme.buttonRadius; implicitHeight: 36 }
            }

            Button {
                text: "Link"
                font.family: Theme.fontFamily
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Theme.primaryHover : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
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
