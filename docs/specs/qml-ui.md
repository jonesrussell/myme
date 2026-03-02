# QML UI Specification

Covers `crates/myme-ui/qml/` (QML files only) and `qml.qrc`.

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-ui/qml/Main.qml` | ApplicationWindow root: sidebar + StackView layout, global models, keyboard shortcuts |
| `crates/myme-ui/qml/Theme.qml` | Singleton: Warm Forge color palette, typography, spacing, responsive breakpoints |
| `crates/myme-ui/qml/Icons.qml` | Singleton: Phosphor icon unicode constants, font loader |
| `crates/myme-ui/qml/AppContext.qml` | Singleton: global app state (currentPage, sidebarExpanded, pageStack ref) |
| `crates/myme-ui/qml/Responsive.qml` | Singleton: responsive layout helpers |
| `crates/myme-ui/qml/qmldir` | Module registration: `MyMeTheme` module with Theme, Icons, AppContext, Responsive singletons |
| `crates/myme-ui/qml/components/Sidebar.qml` | Persistent collapsible sidebar (220px/60px), 9 nav items, Settings at bottom |
| `crates/myme-ui/qml/components/NoteCard.qml` | Keep-style note card with color, pin, labels, checklist |
| `crates/myme-ui/qml/components/RepoCard.qml` | Repository card with branch, status, clone/pull actions |
| `crates/myme-ui/qml/components/WeatherWidget.qml` | Dashboard weather widget |
| `crates/myme-ui/qml/components/WeatherCompact.qml` | Compact weather display |
| `crates/myme-ui/qml/components/EmailWidget.qml` | Dashboard email widget |
| `crates/myme-ui/qml/components/CalendarWidget.qml` | Dashboard calendar widget |
| `crates/myme-ui/qml/components/QuickAddBar.qml` | Quick-add input bar |
| `crates/myme-ui/qml/components/ColorPicker.qml` | Note color picker |
| `crates/myme-ui/qml/pages/WelcomePage.qml` | Dashboard: time-based greeting, stat cards, widget grid |
| `crates/myme-ui/qml/pages/NotePage.qml` | Notes page: Google Keep-style grid, search, filter, CRUD |
| `crates/myme-ui/qml/pages/GmailPage.qml` | Gmail inbox with message list, read/archive/trash actions |
| `crates/myme-ui/qml/pages/CalendarPage.qml` | Calendar events list with time-range filtering |
| `crates/myme-ui/qml/pages/ProjectsPage.qml` | Projects list with repo linking and kanban tasks |
| `crates/myme-ui/qml/pages/ProjectDetailPage.qml` | Project detail view with kanban board |
| `crates/myme-ui/qml/pages/OrganizationsPage.qml` | Organizations list with prospect pipeline |
| `crates/myme-ui/qml/pages/OrganizationDetailPage.qml` | Organization detail with prospects and linked projects |
| `crates/myme-ui/qml/pages/RepoPage.qml` | Repository grid with local + GitHub repos |
| `crates/myme-ui/qml/pages/WorkflowsPage.qml` | GitHub Actions workflow listing |
| `crates/myme-ui/qml/pages/WeatherPage.qml` | Full weather display with 7-day forecast |
| `crates/myme-ui/qml/pages/DevToolsPage.qml` | Developer tools: JWT, encoding, UUID, JSON, hash, time, text chunker |
| `crates/myme-ui/qml/pages/SettingsPage.qml` | App settings: GitHub/Google auth, theme, weather unit |
| `qml.qrc` | Qt resource file bundling all QML, fonts, and assets |

## Interface Signatures

### Theme.qml (Singleton)

```qml
pragma Singleton
QtObject {
    // Mode
    property string mode: "auto"                // "light", "dark", "auto"
    property bool systemDark: Qt.styleHints.colorScheme === Qt.Dark
    property bool isDark: mode === "dark" || (mode === "auto" && systemDark)

    // Typography (Outfit variable font)
    property FontLoader outfitFont: FontLoader { source: "fonts/Outfit-Regular.ttf" }
    readonly property string fontFamily: outfitFont.name
    property int fontSizeSmall: 12
    property int fontSizeNormal: 14
    property int fontSizeMedium: 16
    property int fontSizeLarge: 20
    property int fontSizeXLarge: 24
    property int fontSizeTitle: 32

    // Warm Forge Palette
    property color background: isDark ? "#141414" : "#faf8f5"
    property color surface: isDark ? "#1c1c1c" : "#ffffff"
    property color surfaceAlt: isDark ? "#242424" : "#f5f2ed"
    property color text: isDark ? "#e8e4de" : "#1a1a1a"
    property color textSecondary: isDark ? "#a8a29e" : "#6c757d"
    property color primary: isDark ? "#e5a54b" : "#c08832"      // Amber/Gold
    property color primaryHover: isDark ? "#d4952f" : "#a87528"
    property color primaryText: isDark ? "#141414" : "#ffffff"
    property color border: isDark ? "#2a2a2a" : "#e5e0d8"
    property color success: "#5bb98c"
    property color warning: "#e5a54b"
    property color error: "#e57373"
    property color info: "#64b5f6"

    // Sidebar specific
    property color sidebarBg: isDark ? "#111111" : "#f0ece6"
    property color sidebarHover: isDark ? "#1a1a1a" : "#e5e0d8"
    property color sidebarActive: isDark ? "#252525" : "#e0dbd3"
    property color sidebarActiveIndicator: isDark ? "#e5a54b" : "#c08832"

    // Dimensions
    property int cardRadius: 10
    property int cardPadding: 20
    property int buttonRadius: 8
    property int inputRadius: 8
    property int spacingXs: 4
    property int spacingSm: 8
    property int spacingMd: 16
    property int spacingLg: 24
    property int spacingXl: 32
    property int breakpointSm: 600
    property int breakpointMd: 900
    property int breakpointLg: 1200

    // All color properties have 200ms ColorAnimation Behaviors for smooth transitions
}
```

### AppContext.qml (Singleton)

```qml
pragma Singleton
QtObject {
    property var pageStack: null               // Reference to StackView
    property var weatherModel: null            // Shared WeatherModel
    property var gmailModel: null              // Shared GmailModel
    property var calendarModel: null           // Shared CalendarModel
    property string currentPage: "WelcomePage"
    property bool sidebarExpanded: true

    function goToTopLevelPage(url)             // Clears stack, pushes new page
    function pageUrl(name) -> string           // Qt.resolvedUrl("pages/" + name + ".qml")
}
```

### Icons.qml (Singleton)

```qml
pragma Singleton
QtObject {
    property FontLoader phosphor: FontLoader { source: "fonts/Phosphor.ttf" }
    readonly property string family: phosphor.name

    // 50+ icon constants as unicode strings, e.g.:
    readonly property string house: "\ueb9c"
    readonly property string note: "\uec11"
    readonly property string envelopeSimple: "..."
    readonly property string calendarBlank: "..."
    readonly property string gitBranch: "..."
    // ... etc.
}
```

### Main.qml

```qml
ApplicationWindow {
    id: root
    width: 1200; height: 800; minimumWidth: 480; minimumHeight: 400
    color: Theme.background
    property string currentPage: "WelcomePage"

    // Global models (shared across pages via AppContext)
    WeatherModel { id: weatherModel }
    GmailModel { id: gmailModel }
    CalendarModel { id: calendarModel }

    // 100ms polling timers for each global model (running only when loading)
    Timer { interval: 100; running: weatherModel.loading; onTriggered: weatherModel.poll_channel() }
    Timer { interval: 100; running: gmailModel.loading; onTriggered: gmailModel.poll_channel() }
    Timer { interval: 100; running: calendarModel.loading; onTriggered: calendarModel.poll_channel() }

    function navigateToPage(pageName) {
        root.currentPage = pageName
        AppContext.currentPage = pageName
        AppContext.goToTopLevelPage(AppContext.pageUrl(pageName))
    }

    RowLayout {
        anchors.fill: parent; spacing: 0
        Sidebar { expanded: AppContext.sidebarExpanded; currentPage: root.currentPage; onNavigateTo: navigateToPage(pageName) }
        StackView { id: contentStack; /* slide-fade transitions */ }
    }

    // Keyboard shortcuts: Ctrl+1..8 for nav, Ctrl+, for Settings, Ctrl+B for sidebar toggle
}
```

### Sidebar.qml

```qml
Rectangle {
    property bool expanded: true
    property string currentPage: "WelcomePage"
    signal navigateTo(string pageName)

    width: expanded ? 220 : 60
    Behavior on width { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }

    // 9 navigation items via ListModel:
    // Dashboard, Notes, Gmail, Calendar, Projects, Organizations, Repos, Weather, Dev Tools
    // + Settings at bottom

    // Brand mark: amber "M" square + "MyMe" text (hidden when collapsed)
    // Each nav item: icon + label, active state with left indicator bar
    // Collapse/expand toggle button at bottom
}
```

## Data Flow

### Page Navigation

```
User clicks Sidebar item (or presses Ctrl+1..8)
  -> Sidebar emits navigateTo(pageName)
  -> Main.qml: navigateToPage(pageName)
     -> root.currentPage = pageName
     -> AppContext.currentPage = pageName        (for sidebar highlighting)
     -> AppContext.goToTopLevelPage(url)
        -> pageStack.clear()
        -> pageStack.push(url)                   (slide-fade transition)
  -> Sidebar highlights active item
  -> New page component created, Component.onCompleted fires
  -> Page initializes its model, starts polling
```

### Model Polling Pattern (every page)

```
Page.qml:
  SomeModel { id: model; Component.onCompleted: model.fetch_data() }
  Timer { interval: 100; running: model.loading; repeat: true; onTriggered: model.poll_channel() }

  // model.loading starts true when fetch_data() called
  // Timer runs while loading is true
  // poll_channel() checks for results from Rust background task
  // When result arrives: model updates properties, emits signals
  // loading becomes false, Timer stops
  // QML bindings update UI automatically
```

### Staggered List Animation Pattern

```qml
// Used in: NotePage, GmailPage, CalendarPage, RepoPage, ProjectsPage, WorkflowsPage, OrganizationsPage
delegate: SomeCard {
    opacity: 0
    Component.onCompleted: fadeIn.start()
    SequentialAnimation {
        id: fadeIn
        PauseAnimation { duration: index * 30 }      // 30ms stagger per item
        ParallelAnimation {
            NumberAnimation { target: card; property: "opacity"; to: 1; duration: 200; easing.type: Easing.OutCubic }
            NumberAnimation { target: card; property: "y"; from: card.y + 8; to: card.y; duration: 200; easing.type: Easing.OutCubic }
        }
    }
}
```

## Storage / Schema

No direct storage in QML layer. All persistence goes through Rust models:
- Notes -> NoteModel -> NoteClient (SQLite)
- Gmail -> GmailModel -> GmailClient + GmailCache (API + SQLite)
- Calendar -> CalendarModel -> CalendarClient + CalendarCache (API + SQLite)
- Weather -> WeatherModel -> WeatherProvider + WeatherCache (API + JSON)

### QML Resource Bundle (qml.qrc)

```xml
<!-- All QML files bundled at compile time via qml.qrc -->
<RCC version="1.0">
  <qresource prefix="/">
    <!-- Singletons: Main.qml, Theme.qml, Icons.qml, AppContext.qml, Responsive.qml, qmldir -->
    <!-- Fonts: Phosphor.ttf, Outfit-Regular.ttf -->
    <!-- 13 pages: WelcomePage, NotePage, GmailPage, CalendarPage, ProjectsPage,
         ProjectDetailPage, RepoPage, WorkflowsPage, OrganizationsPage,
         OrganizationDetailPage, SettingsPage, WeatherPage, DevToolsPage -->
    <!-- 9 components: Sidebar, NoteCard, RepoCard, WeatherWidget, WeatherCompact,
         EmailWidget, CalendarWidget, QuickAddBar, ColorPicker -->
    <!-- Assets: icon.png -->
  </qresource>
</RCC>
```

## Configuration

### Theme Modes

| Mode | Behavior |
|------|----------|
| `"auto"` | Follows system `Qt.styleHints.colorScheme` |
| `"dark"` | Always dark palette |
| `"light"` | Always light palette |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+1 | Dashboard |
| Ctrl+2 | Notes |
| Ctrl+3 | Gmail |
| Ctrl+4 | Calendar |
| Ctrl+5 | Projects |
| Ctrl+6 | Organizations |
| Ctrl+7 | Repos |
| Ctrl+8 | Weather |
| Ctrl+, | Settings |
| Ctrl+B | Toggle sidebar |

### QML Module Registration (qmldir)

```
module MyMeTheme
singleton Theme 1.0 Theme.qml
singleton Icons 1.0 Icons.qml
singleton AppContext 1.0 AppContext.qml
singleton Responsive 1.0 Responsive.qml
```

### cxx-qt Model Import

```qml
import myme_ui      // Provides: NoteModel, RepoModel, GmailModel, CalendarModel,
                     // AuthModel, GoogleAuthModel, WeatherModel, WorkflowModel,
                     // OrganizationModel, ProspectModel, ProjectModel, KanbanModel,
                     // EncodingModel, HashModel, JsonModel, JwtModel, TimeModel, UuidModel
```

## Edge Cases

- **Sidebar persistence**: Sidebar is a sibling of StackView in RowLayout, not inside StackView; this prevents re-creation on page changes
- **QML hot reload**: QML changes don't require rebuild -- just restart the app; only Rust bridge changes need `cargo build` + CMake
- **Model snake_case**: cxx-qt exposes Rust methods as-is; QML must use `model.fetch_notes()` not `model.fetchNotes()`
- **Timer optimization**: Polling timers only run while `model.loading` is true; no CPU waste when idle
- **StackView transitions**: Slide-fade with 200ms OutCubic: opacity 0->1 + x offset 20->0
- **Stagger limit**: At 30ms per item, 100 items = 3s stagger; large lists may feel slow
- **Theme transitions**: All color properties animated with 200ms ColorAnimation for smooth light/dark switching
- **Card borders**: Near-invisible: `border.color: Theme.isDark ? "#ffffff08" : "#00000008"` (3% opacity)
- **Font loading**: Outfit font loaded via single FontLoader; `Font.Bold` etc. for weight variants (variable font)
- **Phosphor icons**: Rendered as text with icon font; use `font.family: Icons.family` and `text: Icons.house`
- **Global models**: WeatherModel, GmailModel, CalendarModel instantiated in Main.qml and shared via AppContext for dashboard widgets
- **New QML files**: Must be added to both `qml.qrc` (resource bundling) and referenced in page navigation
- **DevTools pattern**: Each tool is a `Component`; tools array stores metadata; `Loader` switches based on selected tool index
- **Error banners**: Styled with `border.color: "transparent"` / `border.width: 0` for softer appearance
- **Window constraints**: Minimum 480x400; default 1200x800
