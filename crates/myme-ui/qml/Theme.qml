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

    // Color palette
    property color background: isDark ? "#1a1a2e" : "#f8f9fa"
    property color surface: isDark ? "#16213e" : "#ffffff"
    property color surfaceAlt: isDark ? "#1f3460" : "#f1f3f5"
    property color surfaceHover: isDark ? "#253a5e" : "#e9ecef"

    property color text: isDark ? "#e4e6eb" : "#212529"
    property color textSecondary: isDark ? "#a8a8b3" : "#6c757d"
    property color textMuted: isDark ? "#6c6c7a" : "#adb5bd"

    property color border: isDark ? "#2d3a5c" : "#dee2e6"
    property color borderLight: isDark ? "#252f4a" : "#e9ecef"

    property color primary: isDark ? "#6c63ff" : "#5c5ce0"
    property color primaryHover: isDark ? "#7d75ff" : "#4a4acf"
    property color primaryText: "#ffffff"

    property color success: isDark ? "#4ade80" : "#22c55e"
    property color successBg: isDark ? "#1a3a1a" : "#E8F5E9"
    property color warning: isDark ? "#fbbf24" : "#f59e0b"
    property color warningBg: isDark ? "#3a3a1a" : "#FFF8E1"
    property color error: isDark ? "#f87171" : "#ef4444"
    property color errorBg: isDark ? "#3a1a1a" : "#FFEBEE"
    property color info: isDark ? "#60a5fa" : "#3b82f6"
    property color infoBg: isDark ? "#1a2a3a" : "#E3F2FD"

    // Sidebar specific
    property color sidebarBg: isDark ? "#0f0f23" : "#f8f9fa"
    property color sidebarHover: isDark ? "#1a1a3e" : "#e9ecef"
    property color sidebarActive: isDark ? "#252545" : "#e2e4e8"
    property color sidebarBorder: isDark ? "#1a1a3e" : "#e9ecef"

    // Card styling
    property color cardBg: surface
    property color cardBorder: border
    property int cardRadius: 8
    property int cardPadding: 16

    // Input styling
    property color inputBg: isDark ? "#1f3460" : "#ffffff"
    property color inputBorder: isDark ? "#2d3a5c" : "#ced4da"
    property color inputFocus: primary
    property int inputRadius: 6

    // Button styling
    property int buttonRadius: 6
    property int buttonPadding: 12

    // Shadows (simplified for QML)
    property color shadowColor: isDark ? "#00000080" : "#00000020"

    // Typography
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
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on surface {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on surfaceAlt {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on text {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on textSecondary {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on border {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on sidebarBg {
        ColorAnimation {
            duration: transitionDuration
        }
    }
    Behavior on primary {
        ColorAnimation {
            duration: transitionDuration
        }
    }
}
