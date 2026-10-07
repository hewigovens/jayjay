pub type JayResult<T> = Result<T, JayError>;

#[derive(Debug)]
pub enum JayError {
    RepoNotFound { path: String },
    RevNotFound { rev: String },
    Review { message: String },
    Diff { message: String },
    DiffSelectionStale { path: String },
    ConflictEditorStale { path: String },
    FileEditorStale { path: String },
    WorkingCopyStale,
    Internal { message: String },
    Canceled,
}

impl std::fmt::Display for JayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoNotFound { path } => write!(f, "repository not found at {path}"),
            Self::RevNotFound { rev } => write!(f, "revision not found: {rev}"),
            Self::Review { message } => write!(f, "review error: {message}"),
            Self::Diff { message } => write!(f, "diff error: {message}"),
            Self::DiffSelectionStale { path } => write!(f, "{path}: file changed since the diff was rendered — refresh and retry"),
            Self::ConflictEditorStale { path } => write!(f, "{path}: conflict changed since the editor opened — refresh and retry"),
            Self::FileEditorStale { path } => write!(f, "{path}: file changed since the editor opened — refresh and retry"),
            Self::WorkingCopyStale => f.write_str("working copy is stale: its change was rewritten outside this workspace — update the workspace and retry"),
            Self::Internal { message } => f.write_str(message),
            Self::Canceled => f.write_str("canceled"),
        }
    }
}

impl std::error::Error for JayError {}

impl JayError {
    pub fn review(message: impl std::fmt::Display) -> Self {
        Self::Review {
            message: message.to_string(),
        }
    }

    pub fn internal(message: impl std::fmt::Display) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
}
