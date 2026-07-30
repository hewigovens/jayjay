use std::fs;

use gpui::{Modifiers, TestAppContext, VisualTestContext};
use jayjay_gpui::diff::{DiffSelection, SbsSide};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn small_region_offers_only_show_all(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base = (1..=20)
        .map(|line| format!("row {line:02}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(fixture.path.join("context.txt"), &base).expect("write context base");
    run_jj_in(&fixture.path, &["new"]);
    let edited = base
        .replace("row 03", "changed a")
        .replace("row 14", "changed b");
    fs::write(fixture.path.join("context.txt"), edited).expect("write context edit");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "context.txt", cx);

    let region = largest_region(&view, cx);
    assert!(region.line_count <= 10, "fixture keeps regions chunk-sized");
    assert!(
        cx.debug_bounds(selector(format!(
            "diff-context-unified-{}-show-all",
            region.id
        )))
        .is_some(),
        "small region still offers Show all"
    );
    assert!(
        cx.debug_bounds(selector(format!(
            "diff-context-unified-{}-show-10",
            region.id
        )))
        .is_none(),
        "a region that fits one chunk offers no Show 10"
    );
}

#[gpui::test]
fn context_controls_expand_in_unified_and_side_by_side_and_clear_selections(
    cx: &mut TestAppContext,
) {
    let fixture = context_fixture();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "context.txt", cx);

    let original = largest_region(&view, cx);
    let show_ten_id = selector(format!("diff-context-unified-{}-show-10", original.id));
    let show_ten = cx
        .debug_bounds(show_ten_id)
        .expect("unified Show 10 control");
    let unified_show_all_id = selector(format!("diff-context-unified-{}-show-all", original.id));
    assert!(
        cx.debug_bounds(unified_show_all_id).is_some(),
        "unified separator exposes exactly the paired expansion actions"
    );
    view.update_in(cx, |view, _, _| {
        view.set_diff_selection(Some(DiffSelection::start(0, 0, SbsSide::Unified)));
    });

    cx.simulate_click(show_ten.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(
            !view.has_diff_selection(),
            "expansion clears text selection"
        );
        let region = view
            .view_model()
            .read(cx)
            .current_diff
            .as_ref()
            .expect("expanded diff")
            .lines
            .iter()
            .find_map(|line| {
                line.context_region
                    .filter(|region| region.id == original.id)
            })
            .expect("partially expanded region remains");
        assert_eq!(region.line_count, original.line_count - 10);
    });

    view.update_in(cx, |view, _, cx| view.toggle_view_mode(cx));
    settle_visual(cx);
    let old_show_ten_id = selector(format!("diff-context-sbs-old-{}-show-10", original.id));
    assert!(cx.debug_bounds(old_show_ten_id).is_some());
    let new_show_all_id = selector(format!("diff-context-sbs-new-{}-show-all", original.id));
    let show_all = cx
        .debug_bounds(new_show_all_id)
        .expect("new-side Show all control");
    view.update_in(cx, |view, _, cx| {
        view.start_gutter_selection("context.txt".to_owned(), 0, cx);
    });

    cx.simulate_click(show_all.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.gutter_selection(),
            None,
            "expansion clears gutter selection"
        );
        assert!(
            view.view_model()
                .read(cx)
                .current_diff
                .as_ref()
                .expect("fully expanded diff")
                .lines
                .iter()
                .all(|line| !line
                    .context_region
                    .is_some_and(|region| region.id == original.id)),
            "Show all removes the selected region"
        );
    });
}

#[gpui::test]
fn buttons_stay_stable_when_reveals_shrink_a_large_region(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let base = (1..=24)
        .map(|line| format!("row {line:02}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(fixture.path.join("context.txt"), &base).expect("write context base");
    run_jj_in(&fixture.path, &["new"]);
    let edited = base
        .replace("row 03", "changed a")
        .replace("row 21", "changed b");
    fs::write(fixture.path.join("context.txt"), edited).expect("write context edit");
    run_jj_in(&fixture.path, &["st"]);

    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    select_file(&view, "context.txt", cx);

    let region = largest_region(&view, cx);
    assert!(region.line_count > 10 && region.line_count <= 20);
    let show_ten_id = selector(format!("diff-context-unified-{}-show-10", region.id));
    let show_ten = cx
        .debug_bounds(show_ten_id)
        .expect("large region offers Show 10");

    cx.simulate_click(show_ten.center(), Modifiers::default());
    settle_visual(cx);

    let remaining = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .filter_map(|line| line.context_region)
            .find(|candidate| candidate.id == region.id)
            .expect("the reduced region persists")
    });
    assert!(
        remaining.line_count <= 10,
        "the reveal shrank the region below one chunk"
    );
    assert!(
        cx.debug_bounds(show_ten_id).is_some(),
        "a region that loaded large keeps both controls so the view stays stable"
    );
    assert!(
        cx.debug_bounds(selector(format!(
            "diff-context-unified-{}-show-all",
            region.id
        )))
        .is_some()
    );
}
