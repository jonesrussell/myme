import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import myme_ui
import ".."

Page {
    id: organizationsPage
    title: "Organizations"

    property int organizationCount: 0

    background: Rectangle {
        color: Theme.background
    }

    OrganizationModel {
        id: organizationModel
    }

    Timer {
        id: orgPollTimer
        interval: 100
        running: organizationModel.loading
        repeat: true
        onTriggered: organizationModel.poll_channel()
    }

    Connections {
        target: organizationModel
        function onLoadingChanged() {
            if (!organizationModel.loading) {
                organizationsPage.organizationCount = organizationModel.row_count();
            }
        }
        function onOrganizations_changed() {
            organizationsPage.organizationCount = organizationModel.row_count();
        }
    }

    // Stage colors for prospect pipeline bar
    readonly property var stageColors: ({
        lead: "#8a8580",
        qualified: "#64b5f6",
        contacted: "#F59E0B",
        proposal: "#b39ddb",
        negotiation: "#ff8a65",
        won: "#5bb98c",
        lost: "#e57373"
    })

    header: ToolBar {
        background: Rectangle {
            color: "transparent"
        }

        RowLayout {
            anchors.fill: parent
            spacing: Theme.spacingMd

            Label {
                text: "Organizations"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.Bold
                color: Theme.text
                Layout.leftMargin: Theme.spacingLg
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "+ Add Organization"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                Layout.rightMargin: Theme.spacingLg

                contentItem: Label {
                    text: parent.text
                    font: parent.font
                    color: Theme.primaryText
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: parent.hovered ? Theme.primaryHover : Theme.primary
                    radius: Theme.buttonRadius
                    implicitHeight: 36
                    implicitWidth: 160
                }
                onClicked: addOrgDialog.open()
            }
        }
    }

    // Empty state
    ColumnLayout {
        visible: organizationsPage.organizationCount === 0 && !organizationModel.loading
        anchors.centerIn: parent
        spacing: Theme.spacingMd

        Text {
            text: Icons.buildings
            font.family: Icons.family
            font.pixelSize: 48
            color: Theme.textSecondary
            opacity: 0.5
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "No organizations yet"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeLarge
            color: Theme.textSecondary
            Layout.alignment: Qt.AlignHCenter
        }

        Label {
            text: "Add organizations to track clients, partners, and prospects"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeNormal
            color: Theme.textSecondary
            opacity: 0.7
            Layout.alignment: Qt.AlignHCenter
        }

        Button {
            text: "+ Add Organization"
            font.family: Theme.fontFamily
            Layout.alignment: Qt.AlignHCenter
            contentItem: Label {
                text: parent.text
                font: parent.font
                color: Theme.primaryText
                horizontalAlignment: Text.AlignHCenter
            }
            background: Rectangle {
                color: parent.hovered ? Theme.primaryHover : Theme.primary
                radius: Theme.buttonRadius
                implicitHeight: 36
                implicitWidth: 160
            }
            onClicked: addOrgDialog.open()
        }
    }

    // Loading indicator
    BusyIndicator {
        anchors.centerIn: parent
        running: organizationModel.loading
        visible: organizationModel.loading
    }

    // Error banner
    Rectangle {
        visible: organizationModel.error_message !== ""
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMd
        height: errorLabel.implicitHeight + Theme.spacingMd * 2
        color: Theme.errorBg
        radius: Theme.cardRadius
        border.color: "transparent"
        z: 10

        Label {
            id: errorLabel
            anchors.fill: parent
            anchors.margins: Theme.spacingMd
            text: organizationModel.error_message
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            color: Theme.error
            wrapMode: Text.WordWrap
        }
    }

    // Grid of organization cards
    GridView {
        id: orgGrid
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        visible: organizationsPage.organizationCount > 0

        cellWidth: Math.max(320, (width - Theme.spacingMd) / Math.max(1, Math.floor(width / 360)))
        cellHeight: 200

        model: organizationsPage.organizationCount
        clip: true

        delegate: Item {
            id: orgDelegate
            width: orgGrid.cellWidth
            height: orgGrid.cellHeight
            required property int index
            property int orgIndex: index

            // Staggered fade-in animation
            opacity: 0
            Component.onCompleted: fadeIn.start()
            SequentialAnimation {
                id: fadeIn
                PauseAnimation { duration: orgDelegate.orgIndex * 30 }
                ParallelAnimation {
                    NumberAnimation { target: orgDelegate; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                    NumberAnimation { target: orgDelegate; property: "y"; from: orgDelegate.y + 8; to: orgDelegate.y; duration: 200; easing.type: Easing.OutCubic }
                }
            }

            Rectangle {
                anchors.fill: parent
                anchors.margins: Theme.spacingSm
                color: cardMouse.containsMouse ? (Theme.isDark ? Qt.lighter(Theme.cardBg, 1.05) : Qt.darker(Theme.cardBg, 1.02)) : Theme.cardBg
                radius: Theme.cardRadius
                border.color: Theme.cardBorderSubtle
                border.width: 1

                Behavior on color { ColorAnimation { duration: 100 } }

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.cardPadding
                    spacing: Theme.spacingSm

                    // Name row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingSm

                        Text {
                            text: Icons.buildings
                            font.family: Icons.family
                            font.pixelSize: 20
                            color: Theme.primary
                        }

                        Label {
                            text: organizationModel.get_name(orgDelegate.orgIndex)
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeNormal
                            font.weight: Font.Bold
                            color: Theme.text
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        // Prospect count badge
                        Rectangle {
                            visible: {
                                const counts = JSON.parse(organizationModel.get_prospect_counts(orgDelegate.orgIndex));
                                return counts.total > 0;
                            }
                            width: prospectBadgeLabel.implicitWidth + 12
                            height: 22
                            radius: 11
                            color: Theme.isDark ? "#ffffff15" : "#00000010"

                            Label {
                                id: prospectBadgeLabel
                                anchors.centerIn: parent
                                text: {
                                    const counts = JSON.parse(organizationModel.get_prospect_counts(orgDelegate.orgIndex));
                                    return counts.total + " prospects";
                                }
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                color: Theme.textSecondary
                            }
                        }
                    }

                    // Website
                    Label {
                        visible: text !== ""
                        text: organizationModel.get_website(orgDelegate.orgIndex)
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                        color: Theme.primary
                        opacity: 0.8
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    // Contact
                    RowLayout {
                        visible: organizationModel.get_contact_name(orgDelegate.orgIndex) !== ""
                        Layout.fillWidth: true
                        spacing: Theme.spacingXs

                        Text {
                            text: Icons.user
                            font.family: Icons.family
                            font.pixelSize: 14
                            color: Theme.textSecondary
                        }

                        Label {
                            text: {
                                const name = organizationModel.get_contact_name(orgDelegate.orgIndex);
                                const email = organizationModel.get_contact_email(orgDelegate.orgIndex);
                                return email ? `${name} (${email})` : name;
                            }
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            color: Theme.textSecondary
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                    }

                    Item { Layout.fillHeight: true }

                    // Prospect stage bar
                    Rectangle {
                        Layout.fillWidth: true
                        height: 6
                        radius: 3
                        color: Theme.isDark ? "#ffffff10" : "#00000008"

                        Row {
                            anchors.fill: parent
                            spacing: 0

                            Repeater {
                                model: ["lead", "qualified", "contacted", "proposal", "negotiation", "won", "lost"]

                                Rectangle {
                                    property var counts: JSON.parse(organizationModel.get_prospect_counts(orgDelegate.orgIndex))
                                    property int stageCount: counts[modelData] || 0
                                    property int total: counts.total || 1

                                    width: stageCount > 0 ? (parent.width * stageCount / total) : 0
                                    height: parent.height
                                    radius: 3
                                    color: organizationsPage.stageColors[modelData]
                                    visible: stageCount > 0
                                }
                            }
                        }
                    }
                }

                MouseArea {
                    id: cardMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        const orgId = organizationModel.get_id(orgDelegate.orgIndex);
                        const orgName = organizationModel.get_name(orgDelegate.orgIndex);
                        AppContext.pageStack.push(
                            Qt.resolvedUrl("OrganizationDetailPage.qml"),
                            { organizationId: orgId, organizationName: orgName }
                        );
                    }
                }
            }
        }
    }

    // Add Organization Dialog
    Dialog {
        id: addOrgDialog
        title: "Add Organization"
        modal: true
        anchors.centerIn: parent
        width: Math.min(500, parent.width - 40)

        background: Rectangle {
            color: Theme.cardBg
            radius: Theme.cardRadius
            border.color: Theme.borderLight
            border.width: 1
        }

        header: Label {
            text: "Add Organization"
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
                id: orgNameField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "Organization name"
                color: Theme.text
                background: Rectangle {
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: orgNameField.activeFocus ? Theme.primary : (Theme.borderLight)
                    border.width: 1
                }
            }

            Label { text: "Website"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: orgWebsiteField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "https://example.com"
                color: Theme.text
                background: Rectangle {
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: orgWebsiteField.activeFocus ? Theme.primary : (Theme.borderLight)
                    border.width: 1
                }
            }

            Label { text: "Contact Name"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: orgContactNameField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "Primary contact"
                color: Theme.text
                background: Rectangle {
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: orgContactNameField.activeFocus ? Theme.primary : (Theme.borderLight)
                    border.width: 1
                }
            }

            Label { text: "Contact Email"; font.family: Theme.fontFamily; font.pixelSize: Theme.fontSizeSmall; color: Theme.textSecondary }
            TextField {
                id: orgContactEmailField
                Layout.fillWidth: true
                font.family: Theme.fontFamily
                placeholderText: "email@example.com"
                color: Theme.text
                background: Rectangle {
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: orgContactEmailField.activeFocus ? Theme.primary : (Theme.borderLight)
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
                background: Rectangle { color: parent.hovered ? Theme.primaryHover : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 36; implicitWidth: 80 }
            }
        }

        onAccepted: {
            organizationModel.create_organization(
                orgNameField.text,
                "",
                orgWebsiteField.text,
                orgContactNameField.text,
                orgContactEmailField.text,
                "",
                ""
            );
            orgNameField.text = "";
            orgWebsiteField.text = "";
            orgContactNameField.text = "";
            orgContactEmailField.text = "";
        }

        onRejected: {
            orgNameField.text = "";
            orgWebsiteField.text = "";
            orgContactNameField.text = "";
            orgContactEmailField.text = "";
        }
    }

    Component.onCompleted: {
        organizationModel.fetch_organizations();
    }
}
