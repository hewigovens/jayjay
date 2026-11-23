use crate::harness::{
    click, focus_view, install_test_globals, keys, load_selected_change_files, scroll_wheel,
    settle_visual,
};
use gpui::{Entity, Focusable, Modifiers, TestAppContext, VisualTestContext, px};
use jayjay_gpui::app::actions::ToggleIgnoreWhitespace;
use jayjay_gpui::app::{config, menus};
use jayjay_gpui::repo::RepoWindow;
use jj_test::LinearFixture;

/// Working copy with four files, every file's diff shown in one list.
fn open_stack(
    cx: &mut TestAppContext,
) -> (LinearFixture, Entity<RepoWindow>, &mut VisualTestContext) {
    open_stack_with(cx, |_| {})
}

fn open_stack_with(
    cx: &mut TestAppContext,
    edit: impl FnOnce(&LinearFixture),
) -> (LinearFixture, Entity<RepoWindow>, &mut VisualTestContext) {
    let fixture = LinearFixture::build();
    fixture.add_tracked_working_copy_edits();
    edit(&fixture);
    install_test_globals(cx);
    cx.update(|cx| config::update(cx, |config| config.diff.show_all_file_diffs = true));
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (fixture, view, cx)
}

fn selected_path(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_hunk()
            .map(|hunk| hunk.path.clone())
    })
}

#[gpui::test]
fn every_file_shows_until_one_is_picked_from_the_list(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack(cx);
    for path in ["README.md", "feature.txt", "wip1.txt", "wip2.txt"] {
        assert!(
            cx.debug_bounds(crate::harness::selector(format!("stack-file-{path}")))
                .is_some(),
            "{path} is missing from the stack"
        );
    }
    assert!(cx.debug_bounds("show-all-files").is_none());

    click(cx, "file-row-1");
    assert!(
        cx.debug_bounds("diff-stack").is_none(),
        "picking a file did not show it alone"
    );

    click(cx, "show-all-files");
    assert!(cx.debug_bounds("diff-stack").is_some());

    click(cx, "file-row-1");
    focus_view(&view, cx);
    keys(cx, "escape");
    assert!(
        cx.debug_bounds("diff-stack").is_some(),
        "Escape did not return to every file"
    );
}

#[gpui::test]
fn a_click_in_a_stacked_file_selects_it_without_leaving_the_stack(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack(cx);
    assert_eq!(selected_path(&view, cx).as_deref(), Some("README.md"));

    let file = cx
        .debug_bounds("stack-file-wip1.txt")
        .expect("stacked file");
    cx.simulate_click(file.center(), Modifiers::default());
    settle_visual(cx);

    assert_eq!(selected_path(&view, cx).as_deref(), Some("wip1.txt"));
    assert!(cx.debug_bounds("diff-stack").is_some());
}

#[gpui::test]
fn scrolling_the_stack_builds_only_the_files_in_view(cx: &mut TestAppContext) {
    let (_fixture, _view, cx) = open_stack_with(cx, |fixture| {
        let lines: String = (0..600).map(|line| format!("line {line}\n")).collect();
        std::fs::write(fixture.path.join("feature.txt"), lines).expect("write a long file");
    });
    assert!(cx.debug_bounds("stack-file-README.md").is_some());
    assert!(cx.debug_bounds("stack-file-wip2.txt").is_none());

    let stack = cx.debug_bounds("diff-stack").expect("stack");
    scroll_wheel(cx, stack.center(), px(-100_000.));
    settle_visual(cx);

    assert!(
        cx.debug_bounds("stack-file-README.md").is_none(),
        "a file scrolled far out of view was still built"
    );
    let last = cx.debug_bounds("stack-file-wip2.txt").expect("last file");
    assert!(last.bottom() <= stack.bottom() + px(1.));
}

#[gpui::test]
fn stacked_files_switch_to_side_by_side_together(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack_with(cx, |fixture| {
        std::fs::write(fixture.path.join("feature.txt"), "rewritten\n").expect("rewrite a file");
    });
    assert!(cx.debug_bounds("sbs-new-gutter-row-0").is_none());

    view.update(cx, |view, cx| view.toggle_view_mode(cx));
    settle_visual(cx);

    assert!(cx.debug_bounds("diff-stack").is_some());
    assert!(
        cx.debug_bounds("sbs-new-gutter-row-0").is_some(),
        "a stacked file with both sides did not turn side-by-side"
    );
}

#[gpui::test]
fn find_opens_its_bar_above_the_stack(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack(cx);

    view.update(cx, |view, cx| view.open_find(cx));
    settle_visual(cx);

    assert!(cx.debug_bounds("diff-stack").is_some());
    assert!(cx.debug_bounds("find-bar").is_some());
}

#[gpui::test]
fn a_diff_setting_change_reloads_every_stacked_file(cx: &mut TestAppContext) {
    let (_fixture, _view, cx) = open_stack(cx);
    cx.update(|_, cx| menus::install(cx));
    assert!(cx.debug_bounds("stack-loading").is_none());

    cx.update(|_, cx| cx.dispatch_action(&ToggleIgnoreWhitespace));
    settle_visual(cx);

    assert!(
        cx.debug_bounds("stack-loading").is_none(),
        "a stacked file stayed on its loading placeholder"
    );
}

#[gpui::test]
fn a_reviewed_file_hidden_from_the_list_leaves_the_stack_showing(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack(cx);
    cx.update(|_, cx| config::update(cx, |config| config.diff.hide_reviewed_files = true));
    settle_visual(cx);

    view.update(cx, |view, cx| {
        let (change_id, path, identity) = {
            let vm = view.view_model().read(cx);
            let change = vm.selected_change().expect("selected working copy");
            let file = vm.selected_hunk().expect("selected file");
            (
                change.change_id.id.clone(),
                file.path.clone(),
                file.review_identity.clone(),
            )
        };
        view.toggle_reviewed(change_id, path, identity, cx);
    });
    settle_visual(cx);

    assert!(cx.debug_bounds("stack-file-README.md").is_none());
    assert!(
        cx.debug_bounds("diff-stack").is_some(),
        "hiding the reviewed file showed the next one alone"
    );
}

#[gpui::test]
fn marking_a_stacked_file_reviewed_moves_on_without_leaving_the_stack(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack(cx);

    view.update(cx, |view, cx| view.toggle_reviewed_for_selected_files(cx));
    settle_visual(cx);

    assert_ne!(selected_path(&view, cx).as_deref(), Some("README.md"));
    assert!(
        cx.debug_bounds("diff-stack").is_some(),
        "moving on to the next unreviewed file showed it alone"
    );
}

#[gpui::test]
fn taller_rows_keep_the_row_at_the_top_of_the_stack(cx: &mut TestAppContext) {
    let (_fixture, _view, cx) = open_stack_with(cx, |fixture| {
        let lines: String = (0..600).map(|line| format!("line {line}\n")).collect();
        std::fs::write(fixture.path.join("feature.txt"), lines).expect("write a long file");
    });
    let stack = cx.debug_bounds("diff-stack").expect("stack");
    scroll_wheel(cx, stack.center(), px(-3000.));
    settle_visual(cx);
    let top_row = (0..600)
        .find(|row| {
            cx.debug_bounds(crate::harness::selector(format!("diff-content-row-{row}")))
                .is_some_and(|bounds| bounds.top() >= stack.top())
        })
        .expect("a row at the top of the stack");

    cx.update(|_, cx| config::update(cx, |config| config.code_line_spacing = 2.));
    settle_visual(cx);

    let row = cx
        .debug_bounds(crate::harness::selector(format!(
            "diff-content-row-{top_row}"
        )))
        .expect("the row that was at the top is still built");
    assert!((row.top() - stack.top()).abs() < px(60.));
}

#[gpui::test]
fn the_file_scrolled_into_keeps_its_header_at_the_top(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack_with(cx, |fixture| {
        let lines: String = (0..600).map(|line| format!("line {line}\n")).collect();
        std::fs::write(fixture.path.join("feature.txt"), lines).expect("write a long file");
    });
    assert!(cx.debug_bounds("stack-pinned-header").is_none());

    let stack = cx.debug_bounds("diff-stack").expect("stack");
    scroll_wheel(cx, stack.center(), px(-3000.));
    settle_visual(cx);

    let pinned = cx
        .debug_bounds("stack-pinned-header")
        .expect("pinned header");
    assert_eq!(pinned.top(), stack.top());
    cx.simulate_click(pinned.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(selected_path(&view, cx).as_deref(), Some("feature.txt"));
}

#[gpui::test]
fn a_large_diff_folds_in_the_stack_and_opens_alone(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack_with(cx, |fixture| {
        let lines: String = (0..3200).map(|line| format!("line {line}\n")).collect();
        std::fs::write(fixture.path.join("feature.txt"), lines).expect("write a large file");
    });
    assert!(
        cx.debug_bounds("stack-file-wip2.txt").is_some(),
        "the folded file should leave room for the files after it"
    );

    click(cx, "stack-show-diff");
    assert!(cx.debug_bounds("diff-stack").is_none());
    assert_eq!(selected_path(&view, cx).as_deref(), Some("feature.txt"));
}

#[gpui::test]
fn find_in_a_folded_file_opens_it_alone(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_stack_with(cx, |fixture| {
        let lines: String = (0..3200).map(|line| format!("line {line}\n")).collect();
        std::fs::write(fixture.path.join("feature.txt"), lines).expect("write a large file");
    });
    click(cx, "stack-file-feature.txt");
    assert!(cx.debug_bounds("diff-stack").is_some());

    view.update_in(cx, |view, window, cx| {
        view.open_find(cx);
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_input("line 3000");
    settle_visual(cx);

    assert!(view.read_with(cx, |view, _| view.find_match_count()) > 0);
    assert!(
        cx.debug_bounds("diff-stack").is_none(),
        "a match in a folded file has no row to scroll to in the stack"
    );
}
