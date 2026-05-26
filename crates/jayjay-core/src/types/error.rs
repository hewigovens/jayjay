#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("repository not found at {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("{message}")]
    Internal { message: String },
}

pub use CoreError as Error;

impl CoreError {
    pub fn internal(message: impl std::fmt::Display) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
}

pub type CoreResult<T> = Result<T, Error>;
