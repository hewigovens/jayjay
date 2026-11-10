use std::cell::RefCell;

thread_local! {
    // A synchronous action must keep its receipt even if another thread refreshes the shared repo.
    static OPERATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn take_mutation_operation_id() -> Option<String> {
    OPERATION.with(|operation| operation.borrow_mut().take())
}

/// Runs `write` and returns its value with the id of the operation it recorded on this thread, if any.
pub fn capture_mutation_operation_id<T>(write: impl FnOnce() -> T) -> (T, Option<String>) {
    // A receipt left on this worker thread by an earlier action must not pass for this one's.
    take_mutation_operation_id();
    let value = write();
    (value, take_mutation_operation_id())
}

pub(crate) fn record(operation_id: String) {
    OPERATION.with(|operation| *operation.borrow_mut() = Some(operation_id));
}
