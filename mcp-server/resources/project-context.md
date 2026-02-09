# MyMe Project Context

Short summary for MCP/LLM context. See CLAUDE.md for full details.

## Workspace crates

- myme-core, myme-ui, myme-services, myme-auth, myme-integrations, myme-weather, myme-gmail, myme-calendar

## Key paths

- **QML resources**: `qml.qrc` (root), `crates/myme-ui/qml/` (Theme.qml, Main.qml, pages/, components/)
- **cxx-qt models**: `crates/myme-ui/src/models/*.rs`, registered in `crates/myme-ui/build.rs`
- **Sidebar nav**: `crates/myme-ui/qml/components/Sidebar.qml` (navModel ListElement, getNavIcon)
- **StackView / routing**: Main.qml, AppContext.goToTopLevelPage, AppContext.pageUrl(pageName)

## Reminders

- **Invokable naming**: cxx-qt exposes Rust methods as **snake_case** in QML. Use `model.fetch_notes()`, `model.poll_channel()`, not camelCase.
- **Channel pattern**: No `block_on()`. Use mpsc channels; invokable sends request, Timer polls `poll_channel()`, result updates state and signals.
- **New UI page**: Add Page.qml, add model in src/models, register in build.rs, add to qml.qrc, add ListElement and icon in Sidebar.qml.
- **New service client**: Add client in myme-services, export from lib.rs; if UI needed, add model + channel pattern in myme-ui and optionally AppServices.
