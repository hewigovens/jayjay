use crate::harness::{open_repo, rendered_height, settle_visual, zoom_to_max};
use gpui::{
    Entity, Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase, VisualContext,
    VisualTestContext, point, px, size,
};
use jayjay_gpui::repo::RepoWindow;
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
    let short = rendered_height(cx, "description-body");
    assert!(short > px(0.) && short < px(32.));
    assert!(cx.debug_bounds("description-expansion").is_none());
    select(&view, cx, "Multiline");
    let multiline = rendered_height(cx, "description-body");
    assert!(multiline > short * 2. && multiline < px(80.));

    select(&view, cx, "Wrapped");
    let wide = rendered_height(cx, "description-body");
    cx.simulate_resize(size(px(1080.), px(1000.)));
    settle_visual(cx);
    let narrow = rendered_height(cx, "description-body");
    assert!(
        narrow > wide,
        "wrapping must grow the description: {wide:?} -> {narrow:?}"
    );
    assert!(narrow <= px(80.));

    select(&view, cx, "Long");
    let viewport = cx.debug_bounds("description-body").unwrap();
    assert_eq!(viewport.size.height, px(80.));
    let before = cx.debug_bounds("description-text").unwrap().origin.y;
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("description-text").unwrap().origin.y < before);
    assert_eq!(rendered_height(cx, "description-body"), px(80.));

    let toggle = cx
        .debug_bounds("description-expansion")
        .expect("long description can expand");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let expanded = cx.debug_bounds("description-body").unwrap();
    assert_eq!(expanded.size.height, px(320.));
    assert!(rendered_height(cx, "file-row-0") > px(0.));
    let diff_row = cx
        .debug_bounds("diff-content-row-0")
        .expect("diff stays visible");
    assert!(diff_row.origin.y >= expanded.bottom());
    assert!(diff_row.bottom() < px(1000.));
    let before = cx.debug_bounds("description-text").unwrap().origin.y;
    cx.simulate_event(ScrollWheelEvent {
        position: expanded.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
    assert!(cx.debug_bounds("description-text").unwrap().origin.y < before);
    assert_eq!(rendered_height(cx, "description-body"), px(320.));
    let toggle = cx.debug_bounds("description-expansion").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(rendered_height(cx, "description-body"), px(80.));
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(rendered_height(cx, "description-body"), px(320.));

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
    assert_eq!(
        rendered_height(cx, "description-body"),
        px(320.),
        "display prefix changes must preserve expansion"
    );

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
    assert_eq!(
        rendered_height(cx, "description-body"),
        px(320.),
        "rewriting the selected change preserves expansion"
    );

    select(&view, cx, "Wrapped");
    assert!(rendered_height(cx, "description-body") <= px(80.));
    assert_eq!(
        cx.debug_bounds("description-title").unwrap().origin.y,
        cx.debug_bounds("description-body").unwrap().origin.y,
        "switching to another overflowing description should reset its scroll position"
    );

    select(&view, cx, "Short");
    assert_eq!(rendered_height(cx, "description-body"), short);
    zoom_to_max(cx);
    assert!(rendered_height(cx, "description-body") > short);
    select(&view, cx, "Long");
    assert_eq!(rendered_height(cx, "description-body"), px(80.));
    let title = cx.debug_bounds("description-title").unwrap();
    let viewport = cx.debug_bounds("description-body").unwrap();
    assert_eq!(
        title.origin.y, viewport.origin.y,
        "navigation should start at the description's top"
    );
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
    assert!(cx.debug_bounds("description-empty").is_none());
    assert!(cx.debug_bounds("description-expansion").is_none());
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
    assert!(cx.debug_bounds("description-expansion").is_none());
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
