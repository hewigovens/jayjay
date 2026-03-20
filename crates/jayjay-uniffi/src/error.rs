use jayjay_core::CoreError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum JayJayError {
    #[error("repository not found: {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl From<CoreError> for JayJayError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::RepoNotFound { path } => Self::RepoNotFound { path },
            CoreError::RevNotFound { rev } => Self::RevNotFound { rev },
            CoreError::Internal { message } => Self::Internal { message },
        }
    }
}
