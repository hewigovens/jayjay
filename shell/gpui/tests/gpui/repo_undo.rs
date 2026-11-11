use std::sync::Arc;

use crate::harness::*;
use gpui::{Entity, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::actions::{RedoOperation, UndoOperation};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::window::ChangeAction;
use jayjay_gpui::ui::context_menu::ContextAction;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jayjay_gpui::windows::operation_log::OperationLogView;
use jj_test::{LinearFixture, current_op_id, run_jj_in};

fn run_palette_command(
    fixture: &LinearFixture,
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    query: &str,
) {
    let repo_window = view.clone();
    cx.cx.update(|cx| {
        CommandPalette::open(
            fixture.path.display().to_string().into(),
            Some(repo_window),
            cx,
        )
    });
    let window = cx.cx.windows().last().copied().expect("palette window");
    let mut palette = VisualTestContext::from_window(window, &cx.cx);
    settle_visual(&mut palette);
    palette.simulate_input(query);
    keys(&mut palette, "enter");
    settle_visual(cx);
}

fn duplicate_hello(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    let rev = change_with_subject(view, cx, "add hello")
        .selection_revision()
        .to_owned();
    view.update(cx, |view, cx| {
        view.dispatch_context_action(
            ContextAction::Change(Arc::new(ChangeAction::Duplicate { rev })),
            cx,
        );
    });
    settle_visual(cx);
}

fn hello_copies(view: &Entity<RepoWindow>, cx: &VisualTestContext) -> usize {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .filter(|change| change.description.trim() == "add hello")
            .count()
    })
}

fn toast(view: &Entity<RepoWindow>, cx: &VisualTestContext) -> String {
    view.read_with(cx, |view, _| view.toast())
        .map(Into::into)
        .unwrap_or_default()
}

#[gpui::test]
fn undo_and_redo_shortcuts_step_a_rewrite(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    duplicate_hello(&view, cx);
    assert_eq!(hello_copies(&view, cx), 2);
    focus_view(&view, cx);

    keys(cx, "secondary-z");
    assert_eq!(hello_copies(&view, cx), 1);
    assert!(
        toast(&view, cx).starts_with("Undid duplicate"),
        "{}",
        toast(&view, cx)
    );

    keys(cx, "secondary-shift-z");
    assert_eq!(hello_copies(&view, cx), 2);
    assert!(
        toast(&view, cx).starts_with("Redid duplicate"),
        "{}",
        toast(&view, cx)
    );
}

#[gpui::test]
fn undo_shortcut_in_a_focused_input_edits_its_text(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let operation = current_op_id(&fixture.path);

    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_input("draft");
    keys(cx, "secondary-z");
    assert_eq!(summary.read_with(cx, |input, _| input.text()), "");
    keys(cx, "secondary-shift-z");
    assert_eq!(summary.read_with(cx, |input, _| input.text()), "draft");

    keys(cx, "secondary-l");
    let revset = view.read_with(cx, |view, _| view.revset_editor_text());
    cx.simulate_input(" | trunk()");
    keys(cx, "secondary-z");
    assert_eq!(
        view.read_with(cx, |view, _| view.revset_editor_text()),
        revset
    );

    assert_eq!(current_op_id(&fixture.path), operation);
}

#[gpui::test]
fn undo_toast_undoes_its_rewrite_and_nothing_after_the_repo_moved_on(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    duplicate_hello(&view, cx);
    click(cx, "toast-undo");
    assert_eq!(hello_copies(&view, cx), 1);

    duplicate_hello(&view, cx);
    run_jj_in(&fixture.path, &["describe", "-m", "moved on"]);
    let windows = cx.cx.windows().len();
    click(cx, "toast-undo");

    assert_eq!(hello_copies(&view, cx), 2);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        let working_copy = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("working copy");
        assert_eq!(working_copy.description.trim(), "moved on");
    });
    assert_eq!(cx.cx.windows().len(), windows + 1);
    assert!(
        cx.cx
            .windows()
            .last()
            .and_then(|window| window.downcast::<OperationLogView>())
            .is_some()
    );
}

#[gpui::test]
fn undo_after_a_working_copy_snapshot_explains_why_and_keeps_the_file(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let file = fixture.path.join("added.txt");
    std::fs::write(&file, "new edits").unwrap();
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, cx| vm.refresh(false, cx))
    });
    settle_visual(cx);
    let operation = current_op_id(&fixture.path);
    focus_view(&view, cx);

    keys(cx, "secondary-z");
    assert!(file.exists());
    assert_eq!(current_op_id(&fixture.path), operation);
    assert_eq!(toast(&view, cx), jayjay_i18n::repo::undo_snapshot_refused());
    view.read_with(cx, |view, cx| {
        assert!(view.view_model().read(cx).error.is_none());
    });
}

#[gpui::test]
fn palette_undo_steps_the_last_operation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    duplicate_hello(&view, cx);
    run_palette_command(&fixture, &view, cx, "Undo");
    assert_eq!(hello_copies(&view, cx), 1);
}

#[gpui::test]
fn palette_undo_leaves_an_open_file_editor_and_the_repo_alone(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    std::fs::write(fixture.path.join("draft.txt"), "saved\n").expect("write working-copy file");
    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "draft.txt", cx);
    click(cx, "edit-working-copy-file");
    view.update_in(cx, |view, _, cx| {
        view.set_file_editor_content("unsaved\n".to_owned(), cx);
    });
    let operation = current_op_id(&fixture.path);

    run_palette_command(&fixture, &view, cx, "Undo");

    assert!(view.read_with(cx, |view, _| view.file_editor_active()));
    assert_eq!(current_op_id(&fixture.path), operation);
}

#[gpui::test]
fn menu_undo_and_redo_edit_the_focused_text(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    let operation = current_op_id(&fixture.path);
    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    cx.simulate_input("draft");

    cx.dispatch_action(UndoOperation);
    settle_visual(cx);
    assert_eq!(summary.read_with(cx, |input, _| input.text()), "");
    cx.dispatch_action(RedoOperation);
    settle_visual(cx);
    assert_eq!(summary.read_with(cx, |input, _| input.text()), "draft");
    assert_eq!(current_op_id(&fixture.path), operation);
}
