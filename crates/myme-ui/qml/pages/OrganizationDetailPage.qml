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
    property int selectedProspectIndex: -1

    focus: true
    Keys.onEscapePressed: selectedProspectIndex = -1

    // Urgency helpers
    function daysUntil(dateStr) {
        if (!dateStr || dateStr === "") return -1
        var parts = dateStr.split("-")
        if (parts.length < 3) return -1
        var d = new Date(parts[0], parts[1] - 1, parts[2])
        var today = new Date()
        today.setHours(0, 0, 0, 0)
        return Math.ceil((d - today) / 86400000)
    }

    function urgencyColor(days) {
        if (days < 0) return Theme.textSecondary
        if (days <= 7) return "#E57373"
        if (days <= 21) return "#FF8A65"
        if (days <= 60) return "#F59E0B"
        return Theme.textSecondary
    }

    function urgencyLabel(days) {
        if (days < 0) return "Closed"
        if (days === 0) return "Closes today"
        if (days === 1) return "1 day left"
        return days + " days left"
    }

    onCurrentTabChanged: {
        prospectModel.import_result = ""
        if (currentTab === 2) {
            notesArea.text = prospectModel.get_org_notes()
        }
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
                                    return r.imported + " leads imported"
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

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        spacing: 0

                        ScrollView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            Layout.leftMargin: Theme.spacingMd
                            Layout.rightMargin: detailPage.selectedProspectIndex >= 0 ? 0 : Theme.spacingMd
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
                                        id: stageColumn
                                        Layout.preferredWidth: modelData.key === "lead" ? 320 : 220
                                        Layout.fillHeight: true
                                        color: Theme.isDark ? "#05ffffff" : "#03000000"
                                        radius: Theme.cardRadius
                                        border.color: Theme.cardBorderSubtle
                                        border.width: 1

                                        property string stageKey: modelData.key

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
                                                try {
                                                    if (stageColumn.stageKey === "lead") {
                                                        return JSON.parse(prospectModel.lead_prospects_by_urgency());
                                                    }
                                                    return JSON.parse(prospectModel.prospects_for_stage(stageColumn.stageKey));
                                                } catch (e) {
                                                    console.error("[Pipeline] JSON parse error for stage '" + stageColumn.stageKey + "':", e);
                                                    return [];
                                                }
                                            }

                                            delegate: Rectangle {
                                                id: prospectCard
                                                width: ListView.view.width
                                                height: prospectCardLayout.implicitHeight + Theme.spacingSm * 2
                                                radius: Theme.buttonRadius
                                                color: (detailPage.selectedProspectIndex === prospectCard.prospectIndex)
                                                    ? (Theme.isDark ? "#20ffffff" : "#10000000")
                                                    : (prospectMouse.containsMouse
                                                        ? (Theme.isDark ? Qt.lighter(Theme.surface, 1.05) : Qt.darker(Theme.surface, 1.02))
                                                        : Theme.surface)
                                                border.color: (detailPage.selectedProspectIndex === prospectCard.prospectIndex)
                                                    ? Theme.primary
                                                    : (Theme.isDark ? "#08ffffff" : "#08000000")
                                                border.width: 1

                                                property int prospectIndex: modelData
                                                property bool isLead: stageColumn.stageKey === "lead"

                                                // Staggered animation
                                                opacity: 0
                                                Component.onCompleted: prospectFade.start()
                                                SequentialAnimation {
                                                    id: prospectFade
                                                    PauseAnimation { duration: index * 30 }
                                                    NumberAnimation { target: prospectCard; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                                                }

                                                MouseArea {
                                                    id: prospectMouse
                                                    anchors.fill: parent
                                                    hoverEnabled: true
                                                    cursorShape: Qt.PointingHandCursor
                                                    onClicked: {
                                                        if (detailPage.selectedProspectIndex === prospectCard.prospectIndex) {
                                                            detailPage.selectedProspectIndex = -1
                                                        } else {
                                                            detailPage.selectedProspectIndex = prospectCard.prospectIndex
                                                        }
                                                    }
                                                }

                                                ColumnLayout {
                                                    id: prospectCardLayout
                                                    anchors.left: parent.left
                                                    anchors.right: parent.right
                                                    anchors.top: parent.top
                                                    anchors.margins: Theme.spacingSm
                                                    spacing: Theme.spacingXxs

                                                    // --- Lead-specific urgency + budget row ---
                                                    RowLayout {
                                                        visible: prospectCard.isLead
                                                        Layout.fillWidth: true
                                                        spacing: 4

                                                        property int days: detailPage.daysUntil(prospectModel.get_prospect_closing_date(prospectCard.prospectIndex))

                                                        Rectangle {
                                                            visible: parent.days >= 0
                                                            radius: 3
                                                            color: "transparent"
                                                            implicitHeight: urgencyText.implicitHeight + 4
                                                            implicitWidth: urgencyText.implicitWidth + 8
                                                            border.color: detailPage.urgencyColor(parent.days)
                                                            border.width: 1

                                                            Label {
                                                                id: urgencyText
                                                                anchors.centerIn: parent
                                                                text: detailPage.urgencyLabel(parent.parent.days)
                                                                font.family: Theme.fontFamily
                                                                font.pixelSize: 10
                                                                font.weight: Font.Medium
                                                                color: detailPage.urgencyColor(parent.parent.days)
                                                            }
                                                        }

                                                        Item { Layout.fillWidth: true }

                                                        Label {
                                                            visible: text !== ""
                                                            text: prospectModel.get_prospect_value(prospectCard.prospectIndex)
                                                            font.family: Theme.fontFamily
                                                            font.pixelSize: 10
                                                            font.weight: Font.Medium
                                                            color: Theme.primary
                                                        }
                                                    }

                                                    Label {
                                                        text: prospectModel.get_prospect_name(prospectCard.prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.weight: Font.Medium
                                                        color: Theme.text
                                                        elide: prospectCard.isLead ? Text.ElideNone : Text.ElideRight
                                                        wrapMode: prospectCard.isLead ? Text.WordWrap : Text.NoWrap
                                                        maximumLineCount: prospectCard.isLead ? 2 : 1
                                                        Layout.fillWidth: true
                                                    }

                                                    Label {
                                                        visible: !prospectCard.isLead && text !== ""
                                                        text: prospectModel.get_prospect_value(prospectCard.prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: 11
                                                        color: Theme.primary
                                                    }

                                                    Label {
                                                        visible: text !== ""
                                                        text: prospectModel.get_prospect_contact_name(prospectCard.prospectIndex)
                                                        font.family: Theme.fontFamily
                                                        font.pixelSize: 11
                                                        color: Theme.textSecondary
                                                        elide: Text.ElideRight
                                                        Layout.fillWidth: true
                                                    }

                                                    // --- Lead quick-action buttons ---
                                                    RowLayout {
                                                        visible: prospectCard.isLead && prospectMouse.containsMouse
                                                        Layout.fillWidth: true
                                                        spacing: 4

                                                        // Open RFP
                                                        Button {
                                                            text: "🔗 Open RFP"
                                                            visible: prospectModel.get_prospect_source_url(prospectCard.prospectIndex) !== ""
                                                            font.family: Theme.fontFamily
                                                            font.pixelSize: 10
                                                            contentItem: Label {
                                                                text: parent.text
                                                                font: parent.font
                                                                color: Theme.primary
                                                                horizontalAlignment: Text.AlignHCenter
                                                            }
                                                            background: Rectangle {
                                                                color: parent.hovered ? (Theme.isDark ? "#15ffffff" : "#08000000") : "transparent"
                                                                radius: Theme.buttonRadius
                                                                border.color: Theme.isDark ? "#15ffffff" : "#10000000"
                                                                border.width: 1
                                                                implicitHeight: 24
                                                            }
                                                            onClicked: Qt.openUrlExternally(prospectModel.get_prospect_source_url(prospectCard.prospectIndex))
                                                        }

                                                        Item { Layout.fillWidth: true }

                                                        // Qualify
                                                        Button {
                                                            text: "✓ Qualify"
                                                            font.family: Theme.fontFamily
                                                            font.pixelSize: 10
                                                            contentItem: Label {
                                                                text: parent.text
                                                                font: parent.font
                                                                color: "#5BB98C"
                                                                horizontalAlignment: Text.AlignHCenter
                                                            }
                                                            background: Rectangle {
                                                                color: parent.hovered ? "#155BB98C" : "transparent"
                                                                radius: Theme.buttonRadius
                                                                border.color: "#305BB98C"
                                                                border.width: 1
                                                                implicitHeight: 24
                                                                implicitWidth: 62
                                                            }
                                                            onClicked: {
                                                                if (detailPage.selectedProspectIndex === prospectCard.prospectIndex) {
                                                                    detailPage.selectedProspectIndex = -1
                                                                }
                                                                prospectModel.move_prospect(prospectCard.prospectIndex, "qualified")
                                                            }
                                                        }

                                                        // Skip
                                                        Button {
                                                            text: "✗ Skip"
                                                            font.family: Theme.fontFamily
                                                            font.pixelSize: 10
                                                            contentItem: Label {
                                                                text: parent.text
                                                                font: parent.font
                                                                color: "#E57373"
                                                                horizontalAlignment: Text.AlignHCenter
                                                            }
                                                            background: Rectangle {
                                                                color: parent.hovered ? "#15E57373" : "transparent"
                                                                radius: Theme.buttonRadius
                                                                border.color: "#30E57373"
                                                                border.width: 1
                                                                implicitHeight: 24
                                                                implicitWidth: 50
                                                            }
                                                            onClicked: {
                                                                if (detailPage.selectedProspectIndex === prospectCard.prospectIndex) {
                                                                    detailPage.selectedProspectIndex = -1
                                                                }
                                                                prospectModel.move_prospect(prospectCard.prospectIndex, "lost")
                                                            }
                                                        }
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
                        }  // closes ScrollView
                        // Side panel
                        Rectangle {
                            id: sidePanel
                            Layout.preferredWidth: 380
                            Layout.fillHeight: true
                            visible: detailPage.selectedProspectIndex >= 0
                            color: Theme.isDark ? "#0affffff" : "#06000000"
                            border.color: Theme.isDark ? "#15ffffff" : "#10000000"
                            border.width: 1
                            opacity: detailPage.selectedProspectIndex >= 0 ? 1 : 0
                            Behavior on opacity { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 12
                                spacing: 12
                                visible: detailPage.selectedProspectIndex >= 0

                                // Title + close button
                                RowLayout {
                                    Layout.fillWidth: true

                                    Label {
                                        text: detailPage.selectedProspectIndex >= 0
                                            ? prospectModel.get_prospect_name(detailPage.selectedProspectIndex)
                                            : ""
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeNormal
                                        font.weight: Font.Bold
                                        color: Theme.text
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 2
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }

                                    Button {
                                        contentItem: Text {
                                            text: "✕"
                                            font.family: Theme.fontFamily
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
                                        onClicked: detailPage.selectedProspectIndex = -1
                                    }
                                }

                                // Stage selector
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Label {
                                        text: "Stage"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.textSecondary
                                        Layout.preferredWidth: 70
                                    }

                                    ComboBox {
                                        id: stageCombo
                                        Layout.fillWidth: true
                                        model: ["Lead", "Qualified", "Contacted", "Proposal", "Negotiation", "Won", "Lost"]
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall

                                        currentIndex: {
                                            if (detailPage.selectedProspectIndex < 0) return 0
                                            var s = prospectModel.get_prospect_stage(detailPage.selectedProspectIndex)
                                            var cap = s.charAt(0).toUpperCase() + s.slice(1)
                                            var idx = stageCombo.model.indexOf(cap)
                                            return Math.max(0, idx)
                                        }

                                        contentItem: Label {
                                            leftPadding: 8
                                            text: stageCombo.displayText
                                            font: stageCombo.font
                                            color: Theme.text
                                            verticalAlignment: Text.AlignVCenter
                                        }

                                        background: Rectangle {
                                            color: Theme.isDark ? "#0affffff" : "#06000000"
                                            radius: Theme.buttonRadius
                                            border.color: stageCombo.popup.visible ? Theme.primary : (Theme.isDark ? "#15ffffff" : "#10000000")
                                            border.width: 1
                                            implicitHeight: 32
                                        }

                                        onActivated: {
                                            var stageKey = currentText.toLowerCase()
                                            prospectModel.move_prospect(detailPage.selectedProspectIndex, stageKey)
                                        }
                                    }
                                }

                                // Closing date with urgency
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_closing_date(detailPage.selectedProspectIndex) !== ""

                                    property string closingDate: detailPage.selectedProspectIndex >= 0
                                        ? prospectModel.get_prospect_closing_date(detailPage.selectedProspectIndex) : ""
                                    property int days: detailPage.daysUntil(closingDate)

                                    Label {
                                        text: "Closing"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.textSecondary
                                        Layout.preferredWidth: 70
                                    }

                                    ColumnLayout {
                                        spacing: 2
                                        Label {
                                            text: parent.parent.closingDate
                                            font.family: Theme.fontFamily
                                            font.pixelSize: Theme.fontSizeSmall
                                            color: Theme.text
                                        }
                                        Label {
                                            visible: parent.parent.days >= 0
                                            text: "● " + detailPage.urgencyLabel(parent.parent.days)
                                            font.family: Theme.fontFamily
                                            font.pixelSize: 11
                                            color: detailPage.urgencyColor(parent.parent.days)
                                        }
                                    }
                                }

                                // Budget / value
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_value(detailPage.selectedProspectIndex) !== ""

                                    Label {
                                        text: "Budget"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.textSecondary
                                        Layout.preferredWidth: 70
                                    }

                                    Label {
                                        text: detailPage.selectedProspectIndex >= 0
                                            ? prospectModel.get_prospect_value(detailPage.selectedProspectIndex) : ""
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        font.weight: Font.Medium
                                        color: Theme.primary
                                    }
                                }

                                // Contact
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_contact_name(detailPage.selectedProspectIndex) !== ""

                                    Label {
                                        text: "Contact"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.textSecondary
                                        Layout.preferredWidth: 70
                                    }

                                    Label {
                                        text: detailPage.selectedProspectIndex >= 0
                                            ? prospectModel.get_prospect_contact_name(detailPage.selectedProspectIndex) : ""
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.text
                                        Layout.fillWidth: true
                                        elide: Text.ElideRight
                                    }
                                }

                                // Email
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_contact_email(detailPage.selectedProspectIndex) !== ""

                                    Label {
                                        text: "Email"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.textSecondary
                                        Layout.preferredWidth: 70
                                    }

                                    Label {
                                        text: detailPage.selectedProspectIndex >= 0
                                            ? prospectModel.get_prospect_contact_email(detailPage.selectedProspectIndex) : ""
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.primary
                                        Layout.fillWidth: true
                                        elide: Text.ElideRight
                                    }
                                }

                                // Open Source RFP button
                                Button {
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_source_url(detailPage.selectedProspectIndex) !== ""
                                    Layout.fillWidth: true
                                    contentItem: Label {
                                        text: "🔗  Open Source RFP →"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: "#ffffff"
                                        horizontalAlignment: Text.AlignHCenter
                                    }
                                    background: Rectangle {
                                        color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary
                                        radius: Theme.buttonRadius
                                        implicitHeight: 36
                                    }
                                    onClicked: Qt.openUrlExternally(prospectModel.get_prospect_source_url(detailPage.selectedProspectIndex))
                                }

                                // Description
                                Label {
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_description(detailPage.selectedProspectIndex) !== ""
                                    text: "Description"
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSizeSmall
                                    font.weight: Font.Bold
                                    color: Theme.textSecondary
                                }

                                ScrollView {
                                    visible: detailPage.selectedProspectIndex >= 0
                                             && prospectModel.get_prospect_description(detailPage.selectedProspectIndex) !== ""
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    clip: true

                                    Label {
                                        width: parent.width
                                        text: detailPage.selectedProspectIndex >= 0
                                            ? prospectModel.get_prospect_description(detailPage.selectedProspectIndex) : ""
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        color: Theme.text
                                        wrapMode: Text.WordWrap
                                    }
                                }

                                // Edit + Delete row
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Button {
                                        text: "Edit"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        contentItem: Label {
                                            text: parent.text
                                            font: parent.font
                                            color: "#ffffff"
                                            horizontalAlignment: Text.AlignHCenter
                                        }
                                        background: Rectangle {
                                            color: parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary
                                            radius: Theme.buttonRadius
                                            implicitHeight: 32
                                            implicitWidth: 70
                                        }
                                        onClicked: {
                                            editProspectIndex = detailPage.selectedProspectIndex
                                            editProspectDialog.open()
                                        }
                                    }

                                    Item { Layout.fillWidth: true }

                                    Button {
                                        text: "Delete"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: Theme.fontSizeSmall
                                        contentItem: Label {
                                            text: parent.text
                                            font: parent.font
                                            color: "#E57373"
                                            horizontalAlignment: Text.AlignHCenter
                                        }
                                        background: Rectangle {
                                            color: parent.hovered ? "#20E57373" : "transparent"
                                            radius: Theme.buttonRadius
                                            implicitHeight: 32
                                        }
                                        onClicked: {
                                            prospectModel.delete_prospect(detailPage.selectedProspectIndex)
                                            detailPage.selectedProspectIndex = -1
                                        }
                                    }
                                }
                            }
                        }
                    }  // closes outer RowLayout (kanban + side panel)
                }  // closes ColumnLayout (pipelineTab)
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
                            try {
                                return JSON.parse(prospectModel.linked_projects());
                            } catch (e) {
                                console.error("[Pipeline] JSON parse error for linked_projects:", e);
                                return [];
                            }
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
                    anchors.margins: 20
                    spacing: 8

                    Label {
                        text: "Organization Notes"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeNormal
                        font.weight: Font.Bold
                        color: Theme.text
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        TextArea {
                            id: notesArea
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            color: Theme.text
                            wrapMode: TextArea.Wrap
                            placeholderText: "Write notes about this organization..."

                            background: Rectangle {
                                color: Theme.isDark ? "#05ffffff" : "#03000000"
                                radius: Theme.cardRadius
                                border.color: notesArea.activeFocus
                                    ? Theme.primary
                                    : (Theme.isDark ? "#10ffffff" : "#10000000")
                                border.width: 1
                            }

                            onTextChanged: notesDebounce.restart()
                        }
                    }
                }

                Timer {
                    id: notesDebounce
                    interval: 500
                    repeat: false
                    onTriggered: prospectModel.set_org_notes(notesArea.text)
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
        notesArea.text = prospectModel.get_org_notes()
    }
}
