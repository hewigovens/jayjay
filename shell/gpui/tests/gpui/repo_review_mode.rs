use crate::harness::*;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::app::actions::ToggleReviewMode;
use jayjay_gpui::repo::RepoWindow;
use jayjay_review::ReviewMarkSource;
use jj_test::{LinearFixture, run_jj_in};

fn select_change(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, subject: &str) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.description.trim() == subject)
            .unwrap_or_else(|| panic!("missing {subject} change"))
    });
    view.update_in(cx, |view, _, cx| view.select_change(ix, cx));
    settle_visual(cx);
}

fn is_reviewed(view: &Entity<RepoWindow>, cx: &VisualTestContext, path: &str) -> bool {
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let change_id = &vm.selected_change().expect("selected change").change_id.id;
        let hunk = vm
            .files
            .as_ref()
            .and_then(|files| files.iter().find(|hunk| hunk.path == path))
            .unwrap_or_else(|| panic!("missing {path}"));
        view.is_reviewed(change_id, &hunk.path, &hunk.review_identity)
    })
}

fn review_toggle(cx: &mut VisualTestContext) -> Option<bool> {
    match (
        cx.debug_bounds("toggle-review-mode-on").is_some(),
        cx.debug_bounds("toggle-review-mode-off").is_some(),
    ) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        (false, false) => None,
        (true, true) => panic!("two review toggles"),
    }
}

#[gpui::test]
fn review_mode_is_per_change_outside_the_working_copy(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    let mark = cx
        .debug_bounds("review-flat-0")
        .expect("@ keeps its review marks");
    let row = cx.debug_bounds("file-row-0").expect("file row");
    assert!(
        mark.left() > row.center().x,
        "the review mark trails the file name"
    );
    assert_eq!(review_toggle(cx), None, "@ offers no review mode toggle");
    keys(cx, "tab space");
    assert!(cx.debug_bounds("file-reviewed-count").is_some());
    assert!(
        cx.debug_bounds("file-review-progress").is_none(),
        "@ marks pick files to split or commit, not review progress"
    );
    keys(cx, "space");

    select_change(&view, cx, "add feature");
    assert_eq!(review_toggle(cx), Some(false));
    assert!(cx.debug_bounds("review-flat-0").is_none());
    assert!(cx.debug_bounds("mark-reviewed").is_none());
    assert!(cx.debug_bounds("file-reviewed-count").is_none());
    keys(cx, "tab space");
    assert!(
        !is_reviewed(&view, cx, "feature.txt"),
        "Space marked with review mode off"
    );

    let row_top = cx.debug_bounds("file-row-0").expect("file row").top();
    let toggle = cx.debug_bounds("toggle-review-mode-off").expect("toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(review_toggle(cx), Some(true));
    assert!(
        cx.debug_bounds("file-reviewed-count").is_some()
            && cx.debug_bounds("file-review-progress").is_some(),
        "review mode shows progress from the start"
    );
    assert_eq!(
        cx.debug_bounds("file-row-0").expect("file row").top(),
        row_top,
        "the progress bar shifted the file list"
    );
    assert!(cx.debug_bounds("mark-reviewed").is_some());
    keys(cx, "space");
    assert!(is_reviewed(&view, cx, "feature.txt"));

    select_change(&view, cx, "add hello");
    assert_eq!(
        review_toggle(cx),
        Some(false),
        "review mode leaked to another change"
    );
    select_change(&view, cx, "add feature");
    assert_eq!(
        review_toggle(cx),
        Some(true),
        "the change did not keep its review mode"
    );

    cx.dispatch_action(ToggleReviewMode);
    settle_visual(cx);
    assert_eq!(review_toggle(cx), Some(false));
    assert!(cx.debug_bounds("review-flat-0").is_none());
}

#[gpui::test]
fn a_folder_menu_offers_review_marks_only_in_review_mode(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    std::fs::create_dir_all(fixture.path.join("src")).unwrap();
    std::fs::write(fixture.path.join("src/a.txt"), "new file\n").unwrap();
    run_jj_in(&fixture.path, &["commit", "-m", "add src"]);
    let (view, cx) = open_focused(&fixture, cx);
    select_change(&view, cx, "add src");
    let folder_menu = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, cx| view.build_folder_context_menu("src", cx))
    };

    assert!(folder_menu(cx).is_empty());

    view.update(cx, |view, cx| view.toggle_review_mode(cx));
    settle_visual(cx);
    assert_eq!(folder_menu(cx).len(), 1);
}

#[gpui::test]
fn a_change_with_agent_marks_starts_in_review_mode(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    select_change(&view, cx, "add feature");
    let (change_id, identity) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let hunk = &vm.files.as_ref().expect("files loaded")[0];
        (
            vm.selected_change().expect("selected").change_id.id.clone(),
            hunk.review_identity.clone(),
        )
    });
    let store = cx.update(|_, cx| jayjay_gpui::repo::window::shared_review_store(cx));
    store
        .borrow_mut()
        .mark_reviewed_as(
            &change_id,
            "feature.txt",
            &identity,
            None,
            ReviewMarkSource::Agent,
        )
        .expect("agent mark");
    view.update_in(cx, |_, _, cx| cx.notify());
    settle_visual(cx);
    assert_eq!(review_toggle(cx), Some(true));
    assert!(cx.debug_bounds("review-flat-0").is_some());
}
