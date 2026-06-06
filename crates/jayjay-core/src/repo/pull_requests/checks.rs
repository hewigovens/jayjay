use crate::types::ChecksStatus;

/// A single check outcome, normalized across hosts. Each host maps its own raw
/// check/status vocabulary onto these three cases.
pub(super) enum CheckState {
    Success,
    Failure,
    Pending,
}

/// Aggregate per-check outcomes into a single status. This is the one place the
/// precedence lives: no checks → None, any pending → Pending, any failure →
/// Failing, otherwise Passing.
pub(super) fn rollup(checks: impl IntoIterator<Item = CheckState>) -> ChecksStatus {
    let mut seen = false;
    let mut pending = false;
    let mut failure = false;
    for check in checks {
        seen = true;
        match check {
            CheckState::Pending => pending = true,
            CheckState::Failure => failure = true,
            CheckState::Success => {}
        }
    }
    if !seen {
        ChecksStatus::None
    } else if pending {
        ChecksStatus::Pending
    } else if failure {
        ChecksStatus::Failing
    } else {
        ChecksStatus::Passing
    }
}
