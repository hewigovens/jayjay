use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::TestAppContext;
use jayjay_gpui::repo::window::{CommitMessageProvider, StackedPrSnapshot};

use super::{open_stack, open_stack_with_existing_first, settle_visual};

#[derive(Default)]
struct MockAi {
    calls: AtomicUsize,
}

impl CommitMessageProvider for MockAi {
    fn detect(&self) -> Option<String> {
        Some("Codex".to_owned())
    }

    fn generate(&self, _diff_summary: &str) -> Result<String, String> {
        unreachable!("stack naming uses the branch-name provider method")
    }

    fn generate_branch_name(&self, description: &str) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(if description.starts_with("layer one") {
            "ai-base"
        } else {
            "ai-tip"
        }
        .to_owned())
    }
}

#[gpui::test]
fn ai_names_only_auto_assigned_layers_and_preserves_user_edits(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack_with_existing_first(cx, true);
    let StackedPrSnapshot::Preview { names, .. } =
        view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    let ai = Arc::new(MockAi::default());
    view.update_in(cx, |view, _, cx| {
        view.set_commit_message_provider(ai.clone(), cx);
    });
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.generate_stacked_pr_names(cx);
        view.edit_stacked_pr_name(1, "user-tip", cx);
    });
    settle_visual(cx);

    let StackedPrSnapshot::Preview {
        names: generated, ..
    } = view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    assert_eq!(ai.calls.load(Ordering::SeqCst), 1);
    assert_eq!(generated[0], names[0], "existing bookmarks are untouched");
    assert_eq!(generated[1], "user-tip", "user edits win over the AI reply");
}

#[gpui::test]
fn ai_names_auto_assigned_layers_with_change_id_suffixes(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack(cx);
    let StackedPrSnapshot::Preview {
        names: original, ..
    } = view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    let ai = Arc::new(MockAi::default());
    view.update_in(cx, |view, _, cx| view.set_commit_message_provider(ai, cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| view.generate_stacked_pr_names(cx));
    settle_visual(cx);

    let StackedPrSnapshot::Preview { names, .. } =
        view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    let base_suffix = original[0].rsplit_once('-').unwrap().1;
    let tip_suffix = original[1].rsplit_once('-').unwrap().1;
    assert_eq!(names[0], format!("ai-base-{base_suffix}"));
    assert_eq!(names[1], format!("ai-tip-{tip_suffix}"));
}
