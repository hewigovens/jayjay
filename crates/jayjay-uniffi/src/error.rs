use jayjay_core::CoreError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum JayJayError {
    #[error("repository not found: {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("review error: {message}")]
    Review { message: String },
    #[error("diff error: {message}")]
    Diff { message: String },
    #[error("{path}: file changed since the diff was rendered — refresh and retry")]
    DiffSelectionStale { path: String },
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl From<CoreError> for JayJayError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::RepoNotFound { path } => Self::RepoNotFound { path },
            CoreError::RevNotFound { rev } => Self::RevNotFound { rev },
            CoreError::Review { message } => Self::Review { message },
            CoreError::Diff { message } => Self::Diff { message },
            CoreError::DiffSelectionStale { path } => Self::DiffSelectionStale { path },
            CoreError::Internal { message } => Self::Internal { message },
        }
    }
}
