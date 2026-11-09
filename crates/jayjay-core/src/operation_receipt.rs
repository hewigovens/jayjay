use std::cell::RefCell;

thread_local! {
    // A synchronous action must keep its receipt even if another thread refreshes the shared repo.
    static OPERATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn take_mutation_operation_id() -> Option<String> {
    OPERATION.with(|operation| operation.borrow_mut().take())
}

pub(crate) fn record(operation_id: String) {
    OPERATION.with(|operation| *operation.borrow_mut() = Some(operation_id));
}
