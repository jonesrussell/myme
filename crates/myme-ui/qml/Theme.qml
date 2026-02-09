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

    // === Warm Forge Color Palette ===

    // Backgrounds
    property color background: isDark ? "#141414" : "#faf8f5"
    property color surface: isDark ? "#1c1c1c" : "#ffffff"
    property color surfaceAlt: isDark ? "#242424" : "#f5f2ed"
    property color surfaceHover: isDark ? "#2a2a2a" : "#ede8e0"

    // Text
    property color text: isDark ? "#e8e4de" : "#1a1a1a"
    property color textSecondary: isDark ? "#a8a29e" : "#6c757d"
    property color textMuted: isDark ? "#6b6560" : "#adb5bd"

    // Primary (Amber/Gold)
    property color primary: isDark ? "#e5a54b" : "#c08832"
    property color primaryHover: isDark ? "#d4952f" : "#a87528"
    property color primaryText: isDark ? "#141414" : "#ffffff"
    property color primaryGlow: isDark ? "#e5a54b40" : "#c0883240"

    // Borders
    property color border: isDark ? "#2a2a2a" : "#e5e0d8"
    property color borderLight: isDark ? "#333333" : "#ede8e0"

    // Semantic colors
    property color success: "#5bb98c"
    property color successBg: isDark ? "#1a2e22" : "#e8f5ed"
    property color warning: "#e5a54b"
    property color warningBg: isDark ? "#2e2518" : "#fef7e8"
    property color error: "#e57373"
    property color errorBg: isDark ? "#2e1a1a" : "#fdeaea"
    property color info: "#64b5f6"
    property color infoBg: isDark ? "#1a222e" : "#e8f0fe"

    // Sidebar specific
    property color sidebarBg: isDark ? "#111111" : "#f0ece6"
    property color sidebarHover: isDark ? "#1a1a1a" : "#e5e0d8"
    property color sidebarActive: isDark ? "#252525" : "#e0dbd3"
    property color sidebarBorder: isDark ? "#1a1a1a" : "#e5e0d8"
    property color sidebarActiveIndicator: isDark ? "#e5a54b" : "#c08832"

    // Card styling
    property color cardBg: surface
    property color cardBorder: border
    property int cardRadius: 10
    property int cardPadding: 20

    // Input styling
    property color inputBg: isDark ? "#242424" : "#ffffff"
    property color inputBorder: isDark ? "#333333" : "#d5d0c8"
    property color inputFocus: primary
    property int inputRadius: 8

    // Button styling
    property int buttonRadius: 8
    property int buttonPadding: 12

    // Shadows
    property color shadowColor: isDark ? "#00000080" : "#00000015"

    // Typography sizes
    property int fontSizeSmall: 12
    property int fontSizeNormal: 14
    property int fontSizeMedium: 16
    property int fontSizeLarge: 20
    property int fontSizeXLarge: 24
    property int fontSizeTitle: 32

    // Spacing
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
    Behavior on inputBg {
        ColorAnimation { duration: transitionDuration }
    }
}
