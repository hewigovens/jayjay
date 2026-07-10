#[derive(Debug, thiserror::Error)]
pub enum JayJayError {
    #[error("repository not found at {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("review error: {message}")]
    Review { message: String },
    #[error("diff error: {message}")]
    Diff { message: String },
    #[error("{path}: file changed since the diff was rendered — refresh and retry")]
    DiffSelectionStale { path: String },
    #[error("{message}")]
    Internal { message: String },
}

impl JayJayError {
    pub fn review(message: impl std::fmt::Display) -> Self {
        Self::Review {
            message: message.to_string(),
        }
    }

    pub fn diff(message: impl std::fmt::Display) -> Self {
        Self::Diff {
            message: message.to_string(),
        }
    }

    pub fn internal(message: impl std::fmt::Display) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
}
