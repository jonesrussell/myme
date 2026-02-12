pub mod auth_service;
pub mod calendar_service;
pub mod google_common;
pub mod gmail_service;
pub mod kanban_service;
pub mod note_service;
pub mod project_service;
pub mod repo_service;
pub mod weather_service;
pub mod workflow_service;

pub use auth_service::{
    request_authenticate as request_auth, AuthError, AuthServiceMessage,
};
pub use kanban_service::{
    request_create_issue as request_kanban_create, request_sync as request_kanban_sync,
    request_update_issue as request_kanban_update, IssueResult as KanbanIssueResult, KanbanError,
    KanbanServiceMessage,
};
pub use note_service::{
    request_create as request_note_create, request_delete as request_note_delete,
    request_fetch as request_note_fetch, request_toggle_done as request_note_toggle,
    NoteError, NoteServiceMessage,
};
pub use project_service::{
    request_fetch_repo as request_project_fetch_repo, ProjectError, ProjectServiceMessage,
    RepoInfo,
};
pub use repo_service::{request_clone, request_pull, request_refresh, RepoError, RepoServiceMessage};
pub use workflow_service::{
    request_fetch_workflows, RepoWorkflows, WorkflowError, WorkflowServiceMessage,
};
pub use weather_service::{
    request_fetch as request_weather_fetch, WeatherError, WeatherServiceMessage,
};
pub use gmail_service::{
    request_archive as request_gmail_archive, request_fetch as request_gmail_fetch,
    request_mark_as_read as request_gmail_mark_as_read, request_trash as request_gmail_trash,
    GmailError, GmailServiceMessage,
};
pub use calendar_service::{
    request_fetch_events as request_calendar_fetch_events,
    request_fetch_today_events as request_calendar_fetch_today_events,
    CalendarError, CalendarServiceMessage,
};
