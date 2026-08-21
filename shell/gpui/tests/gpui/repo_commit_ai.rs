use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::harness::*;
use gpui::{TestAppContext, VisualTestContext};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::window::CommitMessageProvider;
use jj_test::{LinearFixture, run_jj_in};

struct MockAi {
    name: Option<&'static str>,
    result: Result<&'static str, &'static str>,
    calls: AtomicUsize,
    last_summary: Mutex<String>,
}

impl MockAi {
    fn new(name: Option<&'static str>, result: Result<&'static str, &'static str>) -> Arc<Self> {
        Arc::new(Self {
            name,
            result,
            calls: AtomicUsize::new(0),
            last_summary: Mutex::new(String::new()),
        })
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl CommitMessageProvider for MockAi {
    fn detect(&self) -> Option<String> {
        self.name.map(str::to_owned)
    }

    fn generate(&self, diff_summary: &str) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_summary.lock().unwrap() = diff_summary.to_owned();
        self.result.map(str::to_owned).map_err(str::to_owned)
    }
}

#[gpui::test]
fn generate_fills_commit_box_from_working_copy_diff(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(
        Some("Codex"),
        Ok("Add: capture wip work\n\n- wire the provider"),
    );
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, _| {
        assert_eq!(view.commit_ai_provider_name().as_deref(), Some("Codex"));
    });
    assert!(
        cx.debug_bounds("commit-ai-generate").is_some(),
        "generate button should render in the working-copy commit box"
    );

    view.update_in(cx, |view, _, cx| {
        view.generate_commit_message(cx);
        assert!(view.is_generating_commit_message());
        // A second trigger while in flight must be a no-op, not a queued duplicate.
        view.generate_commit_message(cx);
    });
    settle_visual(cx);

    assert_eq!(mock.calls(), 1);
    assert!(
        mock.last_summary.lock().unwrap().contains("README.md"),
        "provider prompt should include the working-copy diff summary"
    );
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        assert_eq!(
            view.summary_input().read(cx).text(),
            "Add: capture wip work"
        );
        assert_eq!(
            view.description_input().read(cx).text(),
            "- wire the provider"
        );
    });
}

#[gpui::test]
fn generate_button_absent_when_selection_is_not_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(MockAi::new(Some("Codex"), Ok("Add: x")), cx);
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("commit-ai-generate").is_some());

    view.update_in(cx, |view, _, cx| view.select_change(1, cx));
    settle_visual(cx);
    assert!(
        cx.debug_bounds("commit-ai-generate").is_none(),
        "commit box and its generate button should hide off the working copy"
    );
}

#[gpui::test]
fn stale_generation_never_overwrites_user_edits(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(Some("Codex"), Ok("Add: ai text\n\n- ai body"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        view.generate_commit_message(cx);
        // The user keeps typing while the provider is still running.
        view.summary_input()
            .update(cx, |input, cx| input.set_text("my own words", cx));
    });
    settle_visual(cx);

    assert_eq!(mock.calls(), 1, "generation still ran to completion");
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        assert_eq!(
            view.summary_input().read(cx).text(),
            "my own words",
            "the AI reply must be dropped once the user edited mid-generation"
        );
        assert_eq!(view.description_input().read(cx).text(), "");
    });
}

#[gpui::test]
fn commit_mid_generation_drops_the_pending_reply(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(Some("Codex"), Ok("Add: stale ai reply"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        // Generate was triggered on empty fields, then the user typed and committed before the reply landed.
        view.generate_commit_message(cx);
        view.summary_input()
            .update(cx, |input, cx| input.set_text("manual message", cx));
        view.commit_working_copy_from_input(cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        assert!(view.view_model().read(cx).error.is_none());
        assert_eq!(
            view.summary_input().read(cx).text(),
            "",
            "the stale AI reply must not fill the new working copy's cleared commit box"
        );
    });
}

#[gpui::test]
fn describe_mid_generation_keeps_the_pending_reply(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(Some("Codex"), Ok("Add: ai text\n\n- ai body"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        // Unlike commit, describe keeps @ and the box's meaning intact, so it must NOT cancel the in-flight generation; the untouched-snapshot guard alone decides whether the reply lands.
        view.summary_input()
            .update(cx, |input, cx| input.set_text("manual message", cx));
        view.generate_commit_message(cx);
        view.describe_working_copy_from_input(cx);
    });
    settle_visual(cx);

    assert_eq!(mock.calls(), 1);
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "describe errored: {:?}", vm.error);
        let wc = vm
            .graph
            .changes
            .iter()
            .find(|c| c.is_working_copy)
            .expect("working copy after describe");
        assert_eq!(
            wc.description.trim(),
            "manual message",
            "describe applied the text the user had at click time"
        );
        assert_eq!(
            view.summary_input().read(cx).text(),
            "Add: ai text",
            "describe left the snapshot untouched, so the pending reply still fills the box"
        );
        assert_eq!(view.description_input().read(cx).text(), "- ai body");
    });
}

#[gpui::test]
fn generate_without_provider_shows_actionable_toast(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(None, Ok("must never be used"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| view.generate_commit_message(cx));
    settle_visual(cx);

    assert_eq!(mock.calls(), 0);
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        let toast = view.toast().expect("missing-provider toast");
        assert!(
            toast.contains("codex"),
            "toast should name the CLIs: {toast}"
        );
        assert_eq!(view.summary_input().read(cx).text(), "");
    });
}

#[gpui::test]
fn generate_on_empty_working_copy_toasts_without_calling_provider(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    // Fresh empty change on top: nothing to describe.
    run_jj_in(&fixture.path, &["new"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(Some("Codex"), Ok("must never be used"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| view.generate_commit_message(cx));
    settle_visual(cx);

    assert_eq!(mock.calls(), 0);
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        let toast = view.toast().expect("empty working copy toast");
        assert!(
            toast.contains("no changes to describe"),
            "unexpected toast: {toast}"
        );
        assert_eq!(view.summary_input().read(cx).text(), "");
    });
}

#[gpui::test]
fn provider_failure_surfaces_toast_and_keeps_fields(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    let mock = MockAi::new(Some("Claude"), Err("Claude CLI failed"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| view.generate_commit_message(cx));
    settle_visual(cx);

    assert_eq!(mock.calls(), 1);
    view.read_with(cx, |view, cx| {
        assert!(!view.is_generating_commit_message());
        let toast = view.toast().expect("failure toast");
        assert!(
            toast.contains("Claude CLI failed"),
            "unexpected toast: {toast}"
        );
        assert_eq!(view.summary_input().read(cx).text(), "");
        assert_eq!(view.description_input().read(cx).text(), "");
    });
}

#[gpui::test]
fn generate_with_pending_wc_changes_reads_the_current_diff(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["new"]);
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    // Edit after load: the graph still says the working copy is empty.
    std::fs::write(fixture.path.join("late-edit.txt"), "arrived after load\n").expect("write");

    let mock = MockAi::new(Some("Codex"), Ok("Late: describe the fresh edit"));
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(mock.clone(), cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| view.generate_commit_message(cx));
    settle_visual(cx);

    assert_eq!(mock.calls(), 1, "generation must reach the provider");
    assert!(
        mock.last_summary.lock().unwrap().contains("late-edit.txt"),
        "the prompt must contain the post-load edit"
    );
}
