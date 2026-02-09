# Add a new UI page to MyMe

Use this checklist when adding a new top-level page (e.g. **{{pageName}}**). Substitute the page name as needed (PascalCase for the page, snake_case for the model).

## Steps

1. **Create the QML page**  
   - File: `crates/myme-ui/qml/pages/{{pageName}}Page.qml`  
   - Use Theme (import ".."), standard layout. If the page needs data, add a model instance (e.g. `{{modelName}} { id: ... }`).

2. **Create the cxx-qt model**  
   - File: `crates/myme-ui/src/models/{{modelSnake}}_model.rs`  
   - Use `#[cxx_qt::bridge]` and `#[qinvokable]` for methods. **All invokable names must be snake_case** (e.g. `fetch_data()`, `poll_channel()`).  
   - If the page needs async work, use the channel pattern: send requests via mpsc, poll in QML with a Timer calling `poll_channel()`.

3. **Register the model in the build**  
   - In `crates/myme-ui/build.rs`, add:  
     `.file("src/models/{{modelSnake}}_model.rs")`

4. **Add the page to the Qt resource bundle**  
   - In `qml.qrc`, add:  
     `<file>crates/myme-ui/qml/pages/{{pageName}}Page.qml</file>`

5. **Add sidebar navigation**  
   - In `crates/myme-ui/qml/components/Sidebar.qml`:  
     - Add a `ListElement` to the `navModel`: e.g. `ListElement { title: "{{pageTitle}}"; page: "{{pageName}}Page"; icon: "" }`  
     - In `getNavIcon`, add a case for `"{{pageName}}Page"` and return the appropriate `Icons.*` value.

6. **Ensure routing resolves the page**  
   - `AppContext.pageUrl("{{pageName}}Page")` and the StackView should resolve to the new page. If your app uses a central page map, add the new page there.

## Naming conventions

- **Page (PascalCase)**: e.g. Settings → SettingsPage.  
- **Model (snake_case file)**: e.g. settings_model.rs, with type name SettingsModel.  
- **Invokables**: always snake_case in QML (e.g. `model.refresh_data()`).

Reference: CLAUDE.md sections "Adding New UI Pages" and "cxx-qt Invokable Naming".
