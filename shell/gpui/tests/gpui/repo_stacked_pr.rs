mod stacked_pr_ai;
mod stacked_pr_interaction;

use std::sync::{Arc, Mutex};

use crate::harness::*;
use gpui::{TestAppContext, VisualTestContext};
use jayjay_core::{
    CoreResult, Repo, Stack, StackLayerOutcome, StackedPrResult, SubmitStackLayer, SubmittedLayer,
};
use jayjay_gpui::repo::window::StackedPrSnapshot;
use jayjay_gpui::repo::{RepoWindow, StackedPrProvider};
use jj_test::{LinearFixture, run_jj_in};

#[derive(Default)]
struct MockProvider {
    submitted: Mutex<Vec<Vec<SubmitStackLayer>>>,
    existing_first: bool,
}

impl StackedPrProvider for MockProvider {
    fn detect(&self, repo: &Repo, base_rev: &str, tip_rev: &str) -> CoreResult<Stack> {
        assert_eq!(base_rev, "trunk()");
        let mut stack = repo.detect_stack("main", tip_rev)?;
        if self.existing_first {
            stack.layers[0].bookmark_existed = true;
        }
        Ok(stack)
    }

    fn submit(&self, _repo: &Repo, layers: Vec<SubmitStackLayer>) -> CoreResult<StackedPrResult> {
        self.submitted.lock().unwrap().push(layers.clone());
        let mut results: Vec<_> = layers
            .iter()
            .enumerate()
            .map(|(index, layer)| SubmittedLayer {
                bookmark: layer.bookmark.clone(),
                base: if index == 0 {
                    "main".to_owned()
                } else {
                    layers[index - 1].bookmark.clone()
                },
                title: layer.title.clone(),
                outcome: if index == 0 {
                    StackLayerOutcome::Created
                } else {
                    StackLayerOutcome::Updated
                },
                pr_number: (index + 10) as u32,
                pr_url: format!("https://example.test/pr/{}", index + 10),
                detail: "ready".to_owned(),
            })
            .collect();
        results.push(SubmittedLayer {
            bookmark: "failed-layer".to_owned(),
            base: "feature-tip".to_owned(),
            title: "failed layer".to_owned(),
            outcome: StackLayerOutcome::Failed,
            pr_number: 0,
            pr_url: String::new(),
            detail: "forge rejected request".to_owned(),
        });
        Ok(StackedPrResult {
            layers: results,
            message: "Pushed 2 bookmarks".to_owned(),
            open_urls: vec!["https://example.test/pr/11".to_owned()],
        })
    }
}

fn stacked_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["describe", "-m", "layer one\n\nfirst body"],
    );
    run_jj_in(&fixture.path, &["new", "-m", "layer two\n\nsecond body"]);
    fixture
}

fn open_stack(
    cx: &mut TestAppContext,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    Arc<MockProvider>,
    &mut VisualTestContext,
) {
    open_stack_with_existing_first(cx, false)
}

fn open_stack_with_existing_first(
    cx: &mut TestAppContext,
    existing_first: bool,
) -> (
    LinearFixture,
    gpui::Entity<RepoWindow>,
    Arc<MockProvider>,
    &mut VisualTestContext,
) {
    let fixture = stacked_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let provider = Arc::new(MockProvider {
        existing_first,
        ..MockProvider::default()
    });
    view.update_in(cx, |view, _, cx| {
        view.set_stacked_pr_provider(provider.clone());
        view.open_stacked_pr("@".to_owned(), cx);
    });
    settle_visual(cx);
    (fixture, view, provider, cx)
}

#[gpui::test]
fn change_menu_gates_entry_on_mutability(cx: &mut TestAppContext) {
    let fixture = stacked_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let mutable = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| !change.is_immutable)
            .unwrap()
            .clone();
        let mut immutable = mutable.clone();
        immutable.is_immutable = true;
        let labels = |change| {
            view.build_change_menu(change, cx)
                .iter()
                .map(|item| item.label.to_string())
                .collect::<Vec<_>>()
        };
        assert!(
            labels(&mutable)
                .iter()
                .any(|label| label == "Create / Update Stacked PRs…")
        );
        assert!(
            !labels(&immutable)
                .iter()
                .any(|label| label == "Create / Update Stacked PRs…")
        );
    });
}

#[gpui::test]
fn detect_renders_prefilled_bottom_to_top_rows(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack(cx);
    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    let StackedPrSnapshot::Preview {
        names,
        bases,
        can_submit,
        ..
    } = snapshot
    else {
        panic!("expected preview, got {snapshot:?}");
    };
    assert_eq!(names.len(), 2);
    assert!(names[0].starts_with("layer-one-"), "{names:?}");
    assert!(names[1].starts_with("layer-two-"), "{names:?}");
    assert_eq!(bases, vec!["main", names[0].as_str()]);
    assert!(can_submit);
    assert!(cx.debug_bounds("stacked-pr-layer-0").is_some());
    assert!(cx.debug_bounds("stacked-pr-layer-1").is_some());
}

#[gpui::test]
fn reopened_panel_ignores_stale_detection(cx: &mut TestAppContext) {
    let fixture = stacked_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);
    let provider = Arc::new(MockProvider::default());
    view.update_in(cx, |view, _, cx| {
        view.set_stacked_pr_provider(provider);
        view.open_stacked_pr("@".to_owned(), cx);
        view.close_stacked_pr(cx);
        view.open_stacked_pr("@-".to_owned(), cx);
    });

    while cx.background_executor.tick() {
        match view.read_with(cx, |view, _| view.stacked_pr_snapshot()) {
            StackedPrSnapshot::Loading => {}
            StackedPrSnapshot::Preview { names, .. } => assert_eq!(names.len(), 1),
            snapshot => panic!("unexpected snapshot: {snapshot:?}"),
        }
    }
    cx.background_executor.run_until_parked();
    let StackedPrSnapshot::Preview { names, .. } =
        view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    assert_eq!(names.len(), 1);
    assert!(names[0].starts_with("layer-one-"), "{names:?}");
}

#[gpui::test]
fn invalid_and_duplicate_edits_disable_submit_with_warning(cx: &mut TestAppContext) {
    let (_fixture, view, _provider, cx) = open_stack(cx);
    view.update_in(cx, |view, _, cx| {
        view.edit_stacked_pr_name(0, "bad name", cx)
    });
    settle_visual(cx);
    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    let StackedPrSnapshot::Preview {
        warnings,
        can_submit,
        ..
    } = snapshot
    else {
        panic!("expected preview");
    };
    assert_eq!(warnings[0].as_deref(), Some("Not a valid bookmark name"));
    assert!(!can_submit);

    view.update_in(cx, |view, _, cx| {
        view.edit_stacked_pr_name(0, "same-name", cx);
        view.edit_stacked_pr_name(1, "same-name", cx);
    });
    let StackedPrSnapshot::Preview {
        warnings,
        can_submit,
        ..
    } = view.read_with(cx, |view, _| view.stacked_pr_snapshot())
    else {
        panic!("expected preview");
    };
    assert_eq!(
        warnings,
        vec![Some("Duplicate bookmark name".to_owned()); 2]
    );
    assert!(!can_submit);
}

#[gpui::test]
fn edited_names_reach_submit_and_results_render(cx: &mut TestAppContext) {
    let (_fixture, view, provider, cx) = open_stack(cx);
    view.update_in(cx, |view, _, cx| {
        view.edit_stacked_pr_name(0, "feature-base", cx);
        view.edit_stacked_pr_name(1, "feature-tip", cx);
        view.submit_stacked_pr(cx);
    });
    settle_visual(cx);

    {
        let submitted = provider.submitted.lock().unwrap();
        assert_eq!(submitted.len(), 1);
        assert_eq!(submitted[0][0].bookmark, "feature-base");
        assert_eq!(submitted[0][1].bookmark, "feature-tip");
        assert_eq!(submitted[0][0].title, "layer one");
        assert_eq!(submitted[0][1].body, "second body");
    }

    let snapshot = view.read_with(cx, |view, _| view.stacked_pr_snapshot());
    assert_eq!(
        snapshot,
        StackedPrSnapshot::Results {
            outcomes: vec![
                "Created".to_owned(),
                "Updated".to_owned(),
                "Failed".to_owned(),
            ],
            message: "Pushed 2 bookmarks".to_owned(),
        }
    );
    assert!(cx.debug_bounds("stacked-pr-result-0").is_some());
    assert!(cx.debug_bounds("stacked-pr-result-1").is_some());
    assert!(cx.debug_bounds("stacked-pr-result-2").is_some());

    view.update_in(cx, |view, _, cx| view.complete_stacked_pr(cx));
    settle_visual(cx);
    assert_eq!(
        cx.opened_url().as_deref(),
        Some("https://example.test/pr/11")
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.stacked_pr_snapshot()),
        StackedPrSnapshot::Closed
    );
}
