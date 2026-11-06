use jayjay_core::{JayError, error_message};

#[uniffi::export]
pub fn unwrap_command_error(message: String) -> String {
    error_message::unwrap_command_error(&message)
}

#[uniffi::remote(Error)]
pub enum JayError {
    RepoNotFound { path: String },
    RevNotFound { rev: String },
    Review { message: String },
    Diff { message: String },
    DiffSelectionStale { path: String },
    ConflictEditorStale { path: String },
    FileEditorStale { path: String },
    WorkingCopyStale,
    ConvergePlanStale { change_id: String },
    ConvergeBlocked { change_id: String, reason: String },
    ConvergeNotDivergent { change_id: String },
    Refused { message: String },
    Internal { message: String },
    Canceled,
}
