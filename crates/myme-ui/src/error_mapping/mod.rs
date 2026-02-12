//! Maps UI service errors to myme_core::AppError for consistent user-facing messages.
//! Each service has its own module to keep mappings small and readable.

mod auth;
mod calendar;
mod gmail;
mod kanban;
mod note;
mod project;
mod repo;
mod weather;
mod workflow;
