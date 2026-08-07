use super::{ApiError, ReconciliationPlan, Stack, fallback_message, plan_reconciliation};

fn stack(number: u32, pull_requests: &[u32]) -> Stack {
    Stack::from_pull_request_numbers(number, pull_requests)
}

#[test]
fn missing_stack_plans_creation() {
    assert_eq!(
        plan_reconciliation(&[10, 20, 30], None),
        ReconciliationPlan::Create(&[10, 20, 30])
    );
}

#[test]
fn exact_stack_is_unchanged() {
    let existing = stack(7, &[10, 20]);

    assert_eq!(
        plan_reconciliation(&[10, 20], Some(&existing)),
        ReconciliationPlan::Current(7)
    );
}

#[test]
fn append_plan_contains_only_new_top_layers() {
    let existing = stack(7, &[10, 20]);

    assert_eq!(
        plan_reconciliation(&[10, 20, 30, 40], Some(&existing)),
        ReconciliationPlan::Append {
            stack_number: 7,
            pull_requests: &[30, 40],
        }
    );
}

#[test]
fn removal_reorder_and_divergence_are_rejected() {
    let existing = stack(7, &[10, 20]);

    for desired in [&[10][..], &[20, 10], &[10, 30]] {
        assert_eq!(
            plan_reconciliation(desired, Some(&existing)),
            ReconciliationPlan::Diverged(7)
        );
    }
}

#[test]
fn replans_after_a_concurrent_append() {
    let desired = [10, 20, 30];
    let original = stack(7, &[10]);
    assert_eq!(
        plan_reconciliation(&desired, Some(&original)),
        ReconciliationPlan::Append {
            stack_number: 7,
            pull_requests: &[20, 30],
        }
    );

    let updated = stack(7, &[10, 20]);
    assert_eq!(
        plan_reconciliation(&desired, Some(&updated)),
        ReconciliationPlan::Append {
            stack_number: 7,
            pull_requests: &[30],
        }
    );
}

#[test]
fn replans_a_deleted_stack_as_a_full_create() {
    let desired = [10, 20];
    let original = stack(7, &[10]);
    assert!(matches!(
        plan_reconciliation(&desired, Some(&original)),
        ReconciliationPlan::Append { .. }
    ));

    assert_eq!(
        plan_reconciliation(&desired, None),
        ReconciliationPlan::Create(&desired)
    );
}

#[test]
fn fallback_distinguishes_disabled_stacks_from_other_api_errors() {
    let disabled = fallback_message(&ApiError::new(Some(404), "Not Found"));
    assert!(disabled.contains("dependent chain"));
    assert!(disabled.contains("not enabled"));

    let invalid = fallback_message(&ApiError::new(Some(422), "Pull requests must form a stack"));
    assert!(invalid.contains("dependent chain"));
    assert!(invalid.contains("Pull requests must form a stack"));
}
