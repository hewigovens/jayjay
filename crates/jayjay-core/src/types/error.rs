#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("repository not found at {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("{message}")]
    Internal { message: String },
}

pub type CoreResult<T> = Result<T, CoreError>;
