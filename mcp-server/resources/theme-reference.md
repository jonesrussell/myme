# Theme Reference (Warm Forge)

Curated from `crates/myme-ui/qml/Theme.qml`. Update this file when Theme.qml changes.

## Typography

- **Font**: Outfit variable font (`fonts/Outfit-Regular.ttf`). Fallback: Segoe UI.
- **Properties**: `fontFamily`, `fontFamilyMedium`, `fontFamilyBold` (all Outfit when loaded).

## Colors (dark / light)

| Role | Dark | Light |
|------|------|-------|
| background | #141414 | #faf8f5 |
| surface | #1c1c1c | #ffffff |
| surfaceAlt | #242424 | #f5f2ed |
| text | #e8e4de | #1a1a1a |
| textSecondary | #a8a29e | #6c757d |
| primary | #e5a54b | #c08832 |
| primaryHover | #d4952f | #a87528 |
| border | #2a2a2a | #e5e0d8 |
| success | #5bb98c | #5bb98c |
| warning | #e5a54b | #e5a54b |
| error | #e57373 | #e57373 |
| info | #64b5f6 | #64b5f6 |

## Sidebar

- sidebarBg, sidebarHover, sidebarActive, sidebarBorder, sidebarActiveIndicator (see Theme.qml).

## Card & input

- **cardRadius**: 10  
- **cardPadding**: 20  
- **inputRadius**: 8  
- **buttonRadius**: 8  

## Spacing / font sizes

- fontSizeSmall: 12, fontSizeNormal: 14, fontSizeMedium: 16, fontSizeLarge: 20, fontSizeXLarge: 24, fontSizeTitle: 32.
- Spacing tokens in Theme.qml (e.g. spacingSm, spacingMd).

Use `Theme.background`, `Theme.text`, `Theme.primary`, etc. in QML. Prefer theme properties over hardcoded hex in new code.
