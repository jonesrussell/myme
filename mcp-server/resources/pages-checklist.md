# QML Pages and Models Checklist

Curated list of pages and their cxx-qt models. Update when pages or models change.

| Page | QML file | Model (Rust) | In build.rs |
|------|----------|--------------|-------------|
| WelcomePage | pages/WelcomePage.qml | (dashboard) | — |
| NotePage | pages/NotePage.qml | NoteModel | note_model.rs |
| GmailPage | pages/GmailPage.qml | GmailModel | gmail_model.rs |
| CalendarPage | pages/CalendarPage.qml | CalendarModel | calendar_model.rs |
| ProjectsPage | pages/ProjectsPage.qml | ProjectModel | project_model.rs |
| RepoPage | pages/RepoPage.qml | RepoModel | repo_model.rs |
| WeatherPage | pages/WeatherPage.qml | WeatherModel | weather_model.rs |
| DevToolsPage | pages/DevToolsPage.qml | (multiple: JWT, Encoding, UUID, JSON, Hash, Time) | jwt_model, encoding_model, uuid_model, json_model, hash_model, time_model |
| SettingsPage | pages/SettingsPage.qml | GoogleAuthModel, AuthModel | google_auth_model.rs, auth_model.rs |
| WorkflowsPage | pages/WorkflowsPage.qml | WorkflowModel | workflow_model.rs |
| ProjectDetailPage | pages/ProjectDetailPage.qml | ProjectModel | project_model.rs |

**Key paths**

- Pages: `crates/myme-ui/qml/pages/*.qml`
- Models: `crates/myme-ui/src/models/*_model.rs`
- Registration: `crates/myme-ui/build.rs` (`.file("src/models/...")`), `qml.qrc`, `Sidebar.qml` (navModel + getNavIcon).
