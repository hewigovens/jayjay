use crate::harness::{
    install_test_globals, load_selected_change_files, open_repo, rendered_height, settle_visual,
    zoom_to_max,
};
use gpui::{
    Entity, Focusable, Modifiers, Pixels, ScrollDelta, ScrollWheelEvent, TestAppContext,
    TouchPhase, VisualContext, VisualTestContext, point, px, size,
};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::{FocusStop, RepoWindow};
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn description_fits_content_and_scrolls_above_the_cap(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let wrapped = format!("Wrapped {}", "description text ".repeat(8));
    let long = format!("Long\n{}End", "description line\n".repeat(100));
    for (rev, message) in [
        ("subject(\"initial\")", "Short"),
        (
            "subject(\"add hello\")",
            "Multiline\nsecond line\nthird line",
        ),
        ("main", wrapped.as_str()),
        ("@", long.as_str()),
    ] {
        run_jj_in(&fixture.path, &["describe", "-r", rev, "-m", message]);
    }
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    cx.simulate_resize(size(px(1600.), px(1000.)));
    select(&view, cx, "Short");
    let short = rendered_height(cx, "detail-description");
    assert!(short > px(0.) && short < px(46.));
    assert!(
        cx.debug_bounds("description-expansion").is_some(),
        "the expansion toggle shows for any selected change"
    );

    select(&view, cx, "Multiline");
    let multiline = rendered_height(cx, "detail-description");
    assert!(multiline > short);
    let line = rendered_height(cx, "description-body");

    select(&view, cx, "Wrapped");
    let wide = rendered_height(cx, "detail-description");
    cx.simulate_resize(size(px(1080.), px(1000.)));
    settle_visual(cx);
    let narrow = rendered_height(cx, "detail-description");
    assert!(
        narrow > wide,
        "wrapping must grow the description: {wide:?} -> {narrow:?}"
    );
    cx.simulate_resize(size(px(1600.), px(1000.)));
    settle_visual(cx);

    select(&view, cx, "Long");
    let viewport = cx.debug_bounds("description-body").unwrap();
    assert_eq!(viewport.size.height, line);
    let title_before = cx.debug_bounds("description-title").unwrap();
    let before = cx.debug_bounds("description-text").unwrap().origin.y;
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("description-text").unwrap().origin.y < before);
    assert_eq!(cx.debug_bounds("description-title").unwrap(), title_before);
    assert_eq!(rendered_height(cx, "description-body"), line);

    let toggle = cx
        .debug_bounds("description-expansion")
        .expect("long description can expand");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let expanded = cx.debug_bounds("description-body").unwrap();
    assert_snapped_height(cx, 300., line);
    assert!(rendered_height(cx, "file-row-0") > px(0.));
    let diff_row = cx
        .debug_bounds("diff-content-row-0")
        .expect("diff stays visible");
    assert!(diff_row.origin.y >= expanded.bottom());
    assert!(diff_row.bottom() < px(1000.));
    let title_before = cx.debug_bounds("description-title").unwrap();
    let before = cx.debug_bounds("description-text").unwrap().origin.y;
    cx.simulate_event(ScrollWheelEvent {
        position: expanded.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("description-text").unwrap().origin.y < before);
    assert_eq!(cx.debug_bounds("description-title").unwrap(), title_before);
    assert_snapped_height(cx, 300., line);
    let toggle = cx.debug_bounds("description-expansion").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(rendered_height(cx, "description-body"), line);
    let toggle = cx.debug_bounds("description-expansion").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_snapped_height(cx, 300., line);

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| {
            let change = std::sync::Arc::make_mut(&mut vm.graph.changes)
                .iter_mut()
                .find(|change| change.description.starts_with("Long"))
                .unwrap();
            change.commit_id.short_len += 1;
            cx.notify();
        });
    });
    settle_visual(cx);
    assert_snapped_height(cx, 300., line);

    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-r",
            "@",
            "-m",
            &format!("Long rewritten\n{}", "body\n".repeat(100)),
        ],
    );
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx));
    });
    settle_visual(cx);
    assert_snapped_height(cx, 300., line);

    select(&view, cx, "Wrapped");
    assert!(
        cx.debug_bounds("description-body").is_none(),
        "a title-only description renders no body section"
    );
    let title_y = cx.debug_bounds("description-title").unwrap().origin.y;
    let block_y = cx.debug_bounds("detail-description").unwrap().origin.y;
    assert!(
        title_y - block_y <= px(8.),
        "switching changes keeps the title at the top"
    );

    select(&view, cx, "Short");
    assert_eq!(rendered_height(cx, "detail-description"), short);
    zoom_to_max(cx);
    assert!(rendered_height(cx, "detail-description") > short);
    select(&view, cx, "Multiline");
    let zoomed_line = rendered_height(cx, "description-body");
    select(&view, cx, "Long");
    assert_eq!(rendered_height(cx, "description-body"), zoomed_line);
    let viewport = cx.debug_bounds("description-body").unwrap();
    assert_eq!(
        cx.debug_bounds("description-text").unwrap().origin.y,
        viewport.origin.y,
        "navigation should start at the body's top"
    );
}

fn assert_snapped_height(cx: &mut VisualTestContext, cap: f32, line: Pixels) {
    let actual = f32::from(rendered_height(cx, "description-body"));
    let line = f32::from(line);
    assert!(
        actual <= cap + 0.5 && actual > cap - line,
        "description-body: expected whole lines of {line}px just under a {cap}px cap, got {actual}px"
    );
}

#[gpui::test]
fn auto_expand_description_opens_long_messages_expanded(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let long = format!("Long\n{}End", "description line\n".repeat(100));
    let other = format!("Other\n{}End", "description line\n".repeat(100));
    run_jj_in(&fixture.path, &["describe", "-r", "@", "-m", &long]);
    run_jj_in(
        &fixture.path,
        &["describe", "-r", "subject(\"add feature\")", "-m", &other],
    );
    install_test_globals(cx);
    cx.update(|cx| {
        config::update(cx, |c| c.diff.auto_expand_description = true);
    });
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    cx.simulate_resize(size(px(1600.), px(1000.)));
    select(&view, cx, "Long");
    let toggle = cx
        .debug_bounds("description-expansion")
        .expect("long description can collapse");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let line = rendered_height(cx, "description-body");
    let toggle = cx.debug_bounds("description-expansion").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_snapped_height(cx, 300., line);

    select(&view, cx, "Other");
    assert_snapped_height(cx, 300., line);

    select(&view, cx, "add hello");
    assert!(
        cx.debug_bounds("description-expansion").is_some(),
        "the expansion toggle shows even when the body fits"
    );

    select(&view, cx, "Other");
    view.update_in(cx, |_, _, cx| {
        config::update(cx, |c| c.diff.auto_expand_description = false);
    });
    settle_visual(cx);
    assert_eq!(rendered_height(cx, "description-body"), line);
    view.update_in(cx, |_, _, cx| {
        config::update(cx, |c| c.diff.auto_expand_description = true);
    });
    settle_visual(cx);
    assert_snapped_height(cx, 300., line);
    cx.simulate_resize(size(px(1600.), px(600.)));
    settle_visual(cx);
    assert_snapped_height(cx, 180., line);
}

#[gpui::test]
fn description_dialog_keeps_working_copy_draft_and_saves_summary_and_body(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &[
            "describe",
            "-r",
            "subject(\"add hello\")",
            "-m",
            "Editable summary\n    Editable body",
        ],
    );
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    let (working_copy_id, change_count) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        (
            vm.graph
                .changes
                .iter()
                .find(|change| change.is_working_copy)
                .unwrap()
                .change_id
                .id
                .clone(),
            vm.graph.changes.len(),
        )
    });
    view.update_in(cx, |view, _, cx| {
        view.summary_input()
            .update(cx, |input, cx| input.set_text("Keep WIP summary", cx));
        view.description_input()
            .update(cx, |input, cx| input.set_text("Keep WIP body", cx));
    });
    let (summary, body) = view.read_with(cx, |view, _| {
        (
            view.summary_input().clone(),
            view.description_input().clone(),
        )
    });
    assert_tab_cycles_between(cx, &summary, &body);
    select(&view, cx, "Editable summary");
    let original = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .unwrap()
            .description
            .clone()
    });
    let pencil = cx.debug_bounds("edit-description").unwrap();
    cx.simulate_click(pencil.center(), Modifiers::default());
    settle_visual(cx);
    let summary = view.read_with(cx, |view, _| view.text_modal_input().unwrap());
    cx.focus(&summary);
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, cx| view
            .view_model()
            .read(cx)
            .selected_change()
            .unwrap()
            .description
            .clone()),
        original
    );
    let pencil = cx.debug_bounds("edit-description").unwrap();
    cx.simulate_click(pencil.center(), Modifiers::default());
    settle_visual(cx);
    let (summary, body) = view.read_with(cx, |view, _| {
        (
            view.text_modal_input().unwrap(),
            view.text_modal_body_input().unwrap(),
        )
    });
    assert_eq!(
        summary.read_with(cx, |input, _| input.text()),
        "Editable summary"
    );
    assert_eq!(
        body.read_with(cx, |input, _| input.text()),
        "    Editable body"
    );
    assert_tab_cycles_between(cx, &summary, &body);
    summary.update(cx, |input, cx| input.set_text("Cancelled summary", cx));
    body.update(cx, |input, cx| input.set_text("Cancelled body", cx));
    let cancel = cx.debug_bounds("text-modal-cancel").unwrap();
    cx.simulate_click(cancel.center(), Modifiers::default());
    settle_visual(cx);

    cx.simulate_click(pencil.center(), Modifiers::default());
    settle_visual(cx);
    let (summary, body) = view.read_with(cx, |view, _| {
        (
            view.text_modal_input().unwrap(),
            view.text_modal_body_input().unwrap(),
        )
    });
    assert_eq!(
        summary.read_with(cx, |input, _| input.text()),
        "Editable summary"
    );
    assert_eq!(
        body.read_with(cx, |input, _| input.text()),
        "    Editable body"
    );
    summary.update(cx, |input, cx| input.set_text("Updated summary", cx));
    body.update(cx, |input, cx| input.set_text("Updated body", cx));
    cx.focus(&body);
    cx.simulate_keystrokes("enter");
    cx.simulate_input("Second body line");
    assert!(view.read_with(cx, |view, _| view.has_text_modal()));
    cx.focus(&summary);
    cx.simulate_keystrokes("enter");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert!(!view.has_text_modal());
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(
            vm.selected_change().unwrap().description.trim_end(),
            "Updated summary\n\nUpdated body\nSecond body line"
        );
        assert_eq!(
            vm.graph
                .changes
                .iter()
                .find(|change| change.is_working_copy)
                .unwrap()
                .change_id
                .id,
            working_copy_id
        );
        assert_eq!(vm.graph.changes.len(), change_count);
        assert_eq!(view.summary_input().read(cx).text(), "Keep WIP summary");
        assert_eq!(view.description_input().read(cx).text(), "Keep WIP body");
    });
}

#[gpui::test]
fn empty_description_offers_add_only_for_mutable_history(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(
        &fixture.path,
        &["describe", "-r", "subject(\"add hello\")", "-m", ""],
    );
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    view.update_in(cx, |view, _, cx| {
        let ix = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| !change.is_working_copy && change.description.trim().is_empty())
            .unwrap();
        view.view_model()
            .update(cx, |vm, cx| vm.select_change(ix, cx));
    });
    settle_visual(cx);
    let add = cx
        .debug_bounds("edit-description")
        .expect("empty mutable description can be added");
    assert!(
        cx.debug_bounds("description-empty").is_some(),
        "the placeholder shows for any empty description"
    );
    assert!(
        cx.debug_bounds("description-expansion").is_some(),
        "the expansion toggle shows even for an empty description"
    );
    cx.simulate_click(add.center(), Modifiers::default());
    settle_visual(cx);
    let (summary, body) = view.read_with(cx, |view, _| {
        (
            view.text_modal_input().unwrap(),
            view.text_modal_body_input().unwrap(),
        )
    });
    assert!(summary.read_with(cx, |input, _| input.text()).is_empty());
    assert!(body.read_with(cx, |input, _| input.text()).is_empty());
    summary.update(cx, |input, cx| input.set_text("Added description", cx));
    view.update_in(cx, |view, _, cx| view.submit_text_modal(cx));
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model()
                .read(cx)
                .selected_change()
                .unwrap()
                .description
                .trim(),
            "Added description"
        );
    });
    assert!(cx.debug_bounds("description-title").is_some());
    assert!(cx.debug_bounds("description-empty").is_none());

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, cx| {
            let change = std::sync::Arc::make_mut(&mut vm.graph.changes)
                .iter_mut()
                .find(|change| change.description.trim() == "Added description")
                .unwrap();
            change.description.clear();
            change.is_immutable = true;
            cx.notify();
        });
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("description-empty").is_some());
    assert!(cx.debug_bounds("edit-description").is_none());
    assert!(cx.debug_bounds("description-expansion").is_some());
}

#[gpui::test]
fn expansion_toggle_switches_metadata_between_byline_and_grid(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    select(&view, cx, "add hello");
    assert!(cx.debug_bounds("detail-metadata").is_some());
    assert!(cx.debug_bounds("detail-metadata-grid").is_none());

    let toggle = cx
        .debug_bounds("description-expansion")
        .expect("expansion toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("detail-metadata").is_none());
    assert!(cx.debug_bounds("detail-metadata-grid").is_some());

    let toggle = cx.debug_bounds("description-expansion").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert!(cx.debug_bounds("detail-metadata").is_some());
    assert!(cx.debug_bounds("detail-metadata-grid").is_none());
}

#[gpui::test]
fn expand_description_is_a_tab_stop_for_an_empty_description(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx)
    });
    view.read_with(cx, |view, cx| {
        let change = view
            .view_model()
            .read(cx)
            .selected_change()
            .expect("working copy selected");
        assert!(change.description.trim().is_empty());
    });

    for _ in 0..18 {
        if view.read_with(cx, |view, _| view.focused_control())
            == Some(FocusStop::ExpandDescription)
        {
            break;
        }
        cx.simulate_keystrokes("tab");
        settle_visual(cx);
    }
    assert_eq!(
        view.read_with(cx, |view, _| view.focused_control()),
        Some(FocusStop::ExpandDescription)
    );
    cx.simulate_keystrokes("space");
    settle_visual(cx);
    assert!(
        cx.debug_bounds("detail-metadata-grid").is_some(),
        "activating the stop expands the metadata grid"
    );
}

fn assert_tab_cycles_between<T: Focusable + 'static>(
    cx: &mut VisualTestContext,
    summary: &Entity<T>,
    body: &Entity<T>,
) {
    cx.focus(summary);
    cx.simulate_keystrokes("tab");
    assert!(cx.update(|window, cx| body.read(cx).focus_handle(cx).is_focused(window)));
    cx.simulate_keystrokes("shift-tab");
    assert!(cx.update(|window, cx| summary.read(cx).focus_handle(cx).is_focused(window)));
}

fn select(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, subject: &str) {
    view.update_in(cx, |view, _, cx| {
        let ix = view
            .view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.description.starts_with(subject))
            .unwrap();
        view.view_model()
            .update(cx, |vm, cx| vm.select_change(ix, cx));
    });
    settle_visual(cx);
}
