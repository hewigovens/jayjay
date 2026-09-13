use std::sync::Arc;

use super::fixtures::{regions, three_gap_diff};
use super::*;

struct Harness {
    diff: FileDiff,
    old: String,
    new: String,
}

impl Harness {
    fn new() -> Self {
        let (diff, old, new) = three_gap_diff();
        Self { diff, old, new }
    }

    fn source(&self) -> ContextExpansionSource {
        ContextExpansionSource {
            diff: self.diff.clone(),
            old_content: Arc::from(self.old.as_str()),
            new_content: Arc::from(self.new.as_str()),
        }
    }

    fn finish(
        &self,
        session: &mut ContextExpansionSession,
        basis: &str,
        attempt: ContextExpansionAttempt,
    ) -> ContextExpansionFinish {
        let source = attempt.needs_source().then(|| self.source());
        session.finish(basis, attempt.run(source))
    }
}

fn show_more(region_id: u32) -> ContextExpansionRequest {
    ContextExpansionRequest::Region {
        region_id,
        expansion: ContextExpansion::ShowMore { line_count: 10 },
    }
}

fn show_all(region_id: u32) -> ContextExpansionRequest {
    ContextExpansionRequest::Region {
        region_id,
        expansion: ContextExpansion::ShowAll,
    }
}

#[test]
fn a_finished_result_can_be_superseded_before_its_display_projection_installs() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();
    let first = session.begin("a", show_more(region.id)).expect("attempt");
    let first_generation = first.generation();
    assert!(matches!(
        harness.finish(&mut session, "a", first),
        ContextExpansionFinish::Applied { .. }
    ));
    assert!(session.is_current("a", first_generation));
    assert!(!session.is_current("b", first_generation));

    let next = session
        .begin("a", ContextExpansionRequest::AllRegions)
        .expect("attempt");
    let next_generation = next.generation();
    assert!(matches!(
        harness.finish(&mut session, "a", next),
        ContextExpansionFinish::Applied { .. }
    ));
    assert!(session.is_current("a", next_generation));
    assert!(!session.is_current("a", first_generation));

    session.reset();
    assert!(!session.is_current("a", next_generation));
    let replacement = session.begin("a", show_more(region.id)).expect("attempt");
    assert!(session.is_current("a", replacement.generation()));
    assert!(!session.is_current("a", next_generation));
}

#[test]
fn queues_only_the_newest_request_while_one_is_in_flight() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let attempt = session
        .begin("a", show_more(region.id))
        .expect("the first request runs");
    assert!(session.begin("a", show_more(region.id)).is_none());
    assert!(
        session
            .begin("a", ContextExpansionRequest::AllRegions)
            .is_none()
    );

    let ContextExpansionFinish::Applied { next, .. } = harness.finish(&mut session, "a", attempt)
    else {
        panic!("the running request applies");
    };
    assert_eq!(next, Some(ContextExpansionRequest::AllRegions));
}

#[test]
fn a_reset_discards_the_running_attempt_without_disturbing_its_replacement() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let stale = session.begin("a", show_more(region.id)).expect("attempt");
    session.reset();
    let live = session.begin("a", show_more(region.id)).expect("attempt");

    assert!(matches!(
        harness.finish(&mut session, "a", stale),
        ContextExpansionFinish::Discarded
    ));
    assert!(matches!(
        harness.finish(&mut session, "a", live),
        ContextExpansionFinish::Applied { .. }
    ));
}

#[test]
fn a_changed_basis_discards_the_result_and_restarts_from_the_collapsed_diff() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let attempt = session.begin("a", show_more(region.id)).expect("attempt");
    assert!(matches!(
        harness.finish(&mut session, "b", attempt),
        ContextExpansionFinish::Discarded
    ));

    let restarted = session.begin("b", show_more(region.id)).expect("attempt");
    assert!(
        restarted.needs_source(),
        "the document built for the old basis must not be reused"
    );
    assert!(matches!(
        harness.finish(&mut session, "b", restarted),
        ContextExpansionFinish::Applied { .. }
    ));
    assert!(
        !session
            .begin("b", show_more(region.id))
            .expect("attempt")
            .needs_source(),
        "a second expansion of the same basis reuses the built document"
    );
}

#[test]
fn drops_a_queued_request_whose_region_the_running_expansion_consumed() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let attempt = session.begin("a", show_all(region.id)).expect("attempt");
    assert!(session.begin("a", show_more(region.id)).is_none());

    let ContextExpansionFinish::Applied { next, .. } = harness.finish(&mut session, "a", attempt)
    else {
        panic!("the running request applies");
    };
    assert_eq!(next, Some(show_more(region.id)));
    assert!(
        session.begin("a", next.expect("queued request")).is_none(),
        "the fully revealed region is gone"
    );
    assert!(
        session
            .begin("a", ContextExpansionRequest::AllRegions)
            .is_some(),
        "expand-all stays valid while any other region remains"
    );
}

#[test]
fn a_failed_expansion_reports_a_message_and_forgets_the_queue() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let attempt = session.begin("a", show_more(u32::MAX)).expect("attempt");
    assert!(
        session
            .begin("a", ContextExpansionRequest::AllRegions)
            .is_none()
    );
    let ContextExpansionFinish::Failed { message } = harness.finish(&mut session, "a", attempt)
    else {
        panic!("an unknown region fails");
    };
    assert!(message.contains("Refresh"), "{message}");

    let retry = session.begin("a", show_more(region.id)).expect("attempt");
    assert!(
        !retry.needs_source(),
        "the document survives a failed expansion"
    );
    let ContextExpansionFinish::Applied { next, .. } = harness.finish(&mut session, "a", retry)
    else {
        panic!("the retry applies");
    };
    assert_eq!(next, None, "the queue is dropped when a request fails");
}

#[test]
fn applying_reports_the_revealed_lines_and_a_selection_token_that_survives_resets() {
    let harness = Harness::new();
    let region = regions(&harness.diff)[1];
    let mut session = ContextExpansionSession::default();

    let attempt = session.begin("a", show_more(region.id)).expect("attempt");
    let ContextExpansionFinish::Applied {
        reveal,
        selection_generation,
        ..
    } = harness.finish(&mut session, "a", attempt)
    else {
        panic!("the request applies");
    };
    let reveal = reveal.expect("revealed lines");
    assert_eq!(reveal.new_lines.count, 10);
    assert_eq!(
        reveal.new_lines.start,
        region.new_start_line + region.line_count - 10,
        "Show more reveals the suffix nearest the hunk below"
    );

    session.reset();
    let attempt = session.begin("a", show_more(region.id)).expect("attempt");
    let ContextExpansionFinish::Applied {
        selection_generation: after_reset,
        ..
    } = harness.finish(&mut session, "a", attempt)
    else {
        panic!("the request applies");
    };
    assert!(after_reset > selection_generation);
}
