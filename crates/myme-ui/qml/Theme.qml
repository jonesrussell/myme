pragma Singleton
import QtQuick

QtObject {
    id: theme

    // Theme mode: "light", "dark", "auto"
    property string mode: "auto"

    // Detect system dark mode
    property bool systemDark: Qt.styleHints.colorScheme === Qt.Dark

    // Computed dark mode based on settings
    property bool isDark: mode === "dark" || (mode === "auto" && systemDark)

    // Animation duration for theme transitions
    property int transitionDuration: 200

    // === Typography (Outfit variable font) ===
    property FontLoader outfitFont: FontLoader {
        source: "fonts/Outfit-Regular.ttf"
    }

    readonly property string fontFamily: outfitFont.status === FontLoader.Ready ? outfitFont.name : "Segoe UI"
    readonly property string fontFamilyMedium: fontFamily
    readonly property string fontFamilyBold: fontFamily

    // === Indigo/Violet Color Palette ===

    // Backgrounds
    property color background: isDark ? "#131219" : "#F9F8FC"
    property color surface: isDark ? "#1C1A26" : "#FFFFFF"
    property color surfaceAlt: isDark ? "#24223A" : "#F3F2FA"
    property color surfaceHover: isDark ? "#2E2C45" : "#ECEAF5"

    // Text
    property color text: isDark ? "#E4E2F0" : "#1E1B2E"
    property color textSecondary: isDark ? "#9B98B0" : "#6C6982"
    property color textMuted: isDark ? "#5F5C73" : "#A9A6C0"

    // Primary (Indigo/Violet)
    property color primary: isDark ? "#818CF8" : "#6366F1"
    property color primaryHover: isDark ? "#6D78F5" : "#4F46E5"
    property color primaryText: isDark ? "#131219" : "#FFFFFF"
    property color primaryGlow: isDark ? "#818CF840" : "#6366F140"

    // Borders
    property color border: isDark ? "#2E2C45" : "#E2E0EE"
    property color borderLight: isDark ? "#383556" : "#ECEAF5"

    // Semantic colors
    property color success: "#5BB98C"
    property color successBg: isDark ? "#1A2E22" : "#E8F5ED"
    property color warning: "#F59E0B"
    property color warningBg: isDark ? "#2E2518" : "#FEF7E8"
    property color error: isDark ? "#F5A5A5" : "#C53030"
    property color errorBg: isDark ? "#3D2020" : "#FDE8E8"
    property color info: "#64B5F6"
    property color infoBg: isDark ? "#1A222E" : "#E8F0FE"

    // Sidebar specific
    property color sidebarBg: isDark ? "#0F0E16" : "#F0EEF8"
    property color sidebarHover: isDark ? "#1A1826" : "#E2E0EE"
    property color sidebarActive: isDark ? "#252340" : "#DDDAF0"
    property color sidebarBorder: isDark ? "#1A1826" : "#E2E0EE"
    property color sidebarActiveIndicator: isDark ? "#818CF8" : "#6366F1"

    // Card styling
    property color cardBg: surface
    property color cardBorder: border
    property color cardBorderSubtle: isDark ? "#ffffff08" : "#00000008"
    property int cardRadius: 10
    property int cardPadding: 20

    // Input styling
    property color inputBg: isDark ? "#24223A" : "#FFFFFF"
    property color inputBorder: isDark ? "#383556" : "#D3D0E4"
    property color inputFocus: primary
    property int inputRadius: 8

    // Button styling
    property int buttonRadius: 8
    property int buttonPadding: 12

    // Shadows
    property color shadowColor: isDark ? "#00000080" : "#1E1B2E15"

    // Typography sizes
    property int fontSizeSmall: 12
    property int fontSizeNormal: 14
    property int fontSizeMedium: 16
    property int fontSizeLarge: 20
    property int fontSizeXLarge: 24
    property int fontSizeTitle: 32

    // Responsive breakpoints
    property int breakpointSm: 600
    property int breakpointMd: 900
    property int breakpointLg: 1200

    // Spacing
    property int spacingXxs: 2
    property int spacingXs: 4
    property int spacingSm: 8
    property int spacingMd: 16
    property int spacingLg: 24
    property int spacingXl: 32

    // Behavior for smooth color transitions
    Behavior on background {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on surface {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on surfaceAlt {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on surfaceHover {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on text {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on textSecondary {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on textMuted {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on border {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on borderLight {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on sidebarBg {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on sidebarHover {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on sidebarActive {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on primary {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on primaryHover {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on primaryText {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on cardBg {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on cardBorderSubtle {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on inputBg {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on error {
        ColorAnimation { duration: transitionDuration }
    }
    Behavior on errorBg {
        ColorAnimation { duration: transitionDuration }
    }
}
