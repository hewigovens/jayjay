use jayjay_core::commit_message;

/// First line (summary) of a commit message.
#[uniffi::export]
pub fn commit_summary(message: String) -> String {
    commit_message::summary(&message)
}

/// Body — everything after the first line — of a commit message.
#[uniffi::export]
pub fn commit_body(message: String) -> String {
    commit_message::body(&message)
}

/// Combine a summary and an optional body into one commit message.
#[uniffi::export]
pub fn join_commit_message(summary: String, body: String) -> String {
    commit_message::join(&summary, &body)
}

/// Whether the commit-box fields still show the change's own description.
#[uniffi::export]
pub fn commit_draft_is_clean(summary: String, body: String, message: String) -> bool {
    commit_message::draft_is_clean(&summary, &body, &message)
}

/// Apply editor fields without normalizing an unchanged description.
#[uniffi::export]
pub fn update_commit_message(original: String, summary: String, body: String) -> String {
    commit_message::update(&original, &summary, &body)
}
