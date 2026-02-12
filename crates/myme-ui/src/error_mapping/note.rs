use crate::services::note_service::NoteError;
use myme_core::{AppError, DatabaseError};

impl From<NoteError> for AppError {
    fn from(e: NoteError) -> Self {
        match e {
            NoteError::Network(s) => AppError::Database(DatabaseError::QueryFailed(s)),
            NoteError::NotInitialized => AppError::Service("Note service not initialized".into()),
            NoteError::InvalidIndex => AppError::Service("Invalid note index".into()),
        }
    }
}
