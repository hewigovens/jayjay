use std::cell::Cell;
use std::fs;
use std::rc::Rc;

use crate::harness::{install_test_globals, scroll_wheel, scrollable_merge_side, settle_visual};
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, px, size};
use jayjay_core::MergePane;
use jayjay_gpui::external_tool::{ExternalToolInvocation, ExternalToolWindow};

/// The tool leaves the process when jj's contract is met, so the test records that exit instead of performing it.
fn open_tool(
    invocation: ExternalToolInvocation,
    cx: &mut TestAppContext,
) -> (
    Entity<ExternalToolWindow>,
    Rc<Cell<Option<i32>>>,
    &mut VisualTestContext,
) {
    install_test_globals(cx);
    let exit_code = Rc::new(Cell::new(None));
    let recorder = exit_code.clone();
    let (view, cx) = cx.add_window_view(|_, cx| {
        ExternalToolWindow::with_exit(invocation, move |code| recorder.set(Some(code)), cx)
    });
    cx.simulate_resize(size(px(1120.), px(760.)));
    settle_visual(cx);
    (view, exit_code, cx)
}

fn diff_fixture(editable: bool) -> (tempfile::TempDir, ExternalToolInvocation) {
    let fixture = tempfile::tempdir().expect("fixture");
    let left = fixture.path().join("left");
    let right = fixture.path().join("right");
    fs::create_dir(&left).expect("left directory");
    fs::create_dir(&right).expect("right directory");
    fs::write(left.join("file.rs"), "fn value() -> i32 { 1 }\n").expect("left file");
    fs::write(right.join("file.rs"), "fn value() -> i32 { 2 }\n").expect("right file");
    let invocation = ExternalToolInvocation::Diff {
        left: left.to_string_lossy().into_owned(),
        right: right.to_string_lossy().into_owned(),
        editable,
    };
    (fixture, invocation)
}

#[gpui::test]
fn read_only_diff_has_no_edit_or_save_actions(cx: &mut TestAppContext) {
    let (_fixture, invocation) = diff_fixture(false);
    let (_view, _exit_code, cx) = open_tool(invocation, cx);

    assert!(cx.debug_bounds("external-cancel").is_some());
    assert!(cx.debug_bounds("external-file-0").is_some());
    assert!(cx.debug_bounds("external-toggle-file").is_none());
    assert!(cx.debug_bounds("external-save").is_none());
}

#[gpui::test]
fn failed_read_only_diff_closes_with_an_error(cx: &mut TestAppContext) {
    let fixture = tempfile::tempdir().expect("fixture");
    let right = fixture.path().join("right");
    fs::create_dir(&right).expect("right directory");
    let invocation = ExternalToolInvocation::Diff {
        left: fixture
            .path()
            .join("missing-left")
            .to_string_lossy()
            .into_owned(),
        right: right.to_string_lossy().into_owned(),
        editable: false,
    };
    let (view, _exit_code, cx) = open_tool(invocation, cx);

    assert_eq!(view.read_with(cx, |view, _| view.close_exit_code()), 1);
}

#[gpui::test]
fn editable_diff_applies_the_selected_result(cx: &mut TestAppContext) {
    let (fixture, invocation) = diff_fixture(true);
    let (_view, exit_code, cx) = open_tool(invocation, cx);

    let toggle = cx
        .debug_bounds("external-toggle-file")
        .expect("Toggle File button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let save = cx.debug_bounds("external-save").expect("Done button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        exit_code.get(),
        Some(0),
        "a successful save must exit 0 for jj"
    );

    assert_eq!(
        fs::read_to_string(fixture.path().join("right/file.rs")).expect("edited output"),
        "fn value() -> i32 { 1 }\n"
    );
}

#[gpui::test]
fn editable_diff_can_restore_a_binary_file(cx: &mut TestAppContext) {
    let fixture = tempfile::tempdir().expect("fixture");
    let left = fixture.path().join("left");
    let right = fixture.path().join("right");
    fs::create_dir(&left).expect("left directory");
    fs::create_dir(&right).expect("right directory");
    fs::write(left.join("data.bin"), [0, 1, 2]).expect("left binary");
    fs::write(right.join("data.bin"), [0, 3, 4]).expect("right binary");
    let invocation = ExternalToolInvocation::Diff {
        left: left.to_string_lossy().into_owned(),
        right: right.to_string_lossy().into_owned(),
        editable: true,
    };
    let (_view, exit_code, cx) = open_tool(invocation, cx);

    let toggle = cx
        .debug_bounds("external-toggle-file")
        .expect("whole-file toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let save = cx.debug_bounds("external-save").expect("Done button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        exit_code.get(),
        Some(0),
        "a successful save must exit 0 for jj"
    );

    assert_eq!(
        fs::read(right.join("data.bin")).expect("edited output"),
        [0, 1, 2]
    );
}

#[gpui::test]
fn editable_diff_toggles_a_topology_transition_as_one_group(cx: &mut TestAppContext) {
    let fixture = tempfile::tempdir().expect("fixture");
    let left = fixture.path().join("left");
    let right = fixture.path().join("right");
    fs::create_dir(&left).expect("left directory");
    fs::create_dir_all(right.join("item")).expect("right directory");
    fs::write(left.join("item"), "old file\n").expect("left file");
    fs::write(right.join("item/new.txt"), "new file\n").expect("right file");
    let invocation = ExternalToolInvocation::Diff {
        left: left.to_string_lossy().into_owned(),
        right: right.to_string_lossy().into_owned(),
        editable: true,
    };
    let (_view, exit_code, cx) = open_tool(invocation, cx);

    let toggle = cx
        .debug_bounds("external-toggle-file")
        .expect("topology-group toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let save = cx.debug_bounds("external-save").expect("Done button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        exit_code.get(),
        Some(0),
        "a successful save must exit 0 for jj"
    );

    assert!(right.join("item").is_file());
    assert_eq!(
        fs::read_to_string(right.join("item")).expect("restored file"),
        "old file\n"
    );
}

#[cfg(unix)]
#[gpui::test]
fn executable_only_diff_can_restore_the_left_mode(cx: &mut TestAppContext) {
    use std::os::unix::fs::PermissionsExt as _;

    let (fixture, invocation) = diff_fixture(true);
    let left = fixture.path().join("left/file.rs");
    let right = fixture.path().join("right/file.rs");
    fs::write(&right, fs::read(&left).expect("left content")).expect("matching content");
    fs::set_permissions(&left, fs::Permissions::from_mode(0o644)).expect("left mode");
    fs::set_permissions(&right, fs::Permissions::from_mode(0o755)).expect("right mode");
    let (_view, exit_code, cx) = open_tool(invocation, cx);

    let toggle = cx
        .debug_bounds("external-toggle-file")
        .expect("mode-only Toggle File button");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
    let save = cx.debug_bounds("external-save").expect("Done button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        exit_code.get(),
        Some(0),
        "a successful save must exit 0 for jj"
    );

    assert_eq!(
        fs::metadata(right)
            .expect("right metadata")
            .permissions()
            .mode()
            & 0o111,
        0
    );
}

fn merge_fixture() -> (tempfile::TempDir, ExternalToolInvocation, String) {
    let fixture = tempfile::tempdir().expect("fixture");
    let left = fixture.path().join("left.rs");
    let base = fixture.path().join("base.rs");
    let right = fixture.path().join("right.rs");
    let output = fixture.path().join("output.rs");
    fs::write(&left, scrollable_merge_side(10)).expect("left");
    fs::write(&base, scrollable_merge_side(0)).expect("base");
    fs::write(&right, scrollable_merge_side(20)).expect("right");
    fs::write(&output, "").expect("output");
    let invocation = ExternalToolInvocation::Merge {
        left: left.to_string_lossy().into_owned(),
        base: base.to_string_lossy().into_owned(),
        right: right.to_string_lossy().into_owned(),
        output: output.to_string_lossy().into_owned(),
        path: "src/value.rs".to_owned(),
        marker_length: 7,
    };
    (fixture, invocation, scrollable_merge_side(20))
}

#[gpui::test]
fn merge_tool_accepts_a_hunk_and_saves_the_output(cx: &mut TestAppContext) {
    let (fixture, invocation, right_content) = merge_fixture();
    let output = fixture.path().join("output.rs");
    let (view, exit_code, cx) = open_tool(invocation, cx);

    let second_before = cx.debug_bounds("merge-hunk-1").expect("second merge hunk");
    let use_first_right = cx
        .debug_bounds("external-hunk-0-Accept Right")
        .expect("first Accept Right hunk action");
    cx.simulate_click(use_first_right.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view.selected_merge_hunk()),
        1,
        "accepting a side advances the selection to the next unresolved conflict"
    );
    let second_after = cx
        .debug_bounds("merge-hunk-1")
        .expect("revealed second merge hunk");
    assert!(
        second_after.origin.y < second_before.origin.y,
        "advancing the selection should reveal the next card"
    );
    let use_second_right = cx
        .debug_bounds("external-hunk-1-Accept Right")
        .expect("second Accept Right hunk action");
    cx.simulate_click(use_second_right.center(), Modifiers::default());
    settle_visual(cx);
    let save = cx.debug_bounds("external-save").expect("Done button");
    cx.simulate_click(save.center(), Modifiers::default());
    settle_visual(cx);
    assert_eq!(
        exit_code.get(),
        Some(0),
        "a successful save must exit 0 for jj"
    );

    assert_eq!(
        fs::read_to_string(output).expect("merge output"),
        right_content
    );
}

#[gpui::test]
fn merge_panes_stay_aligned_and_track_the_visible_conflict(cx: &mut TestAppContext) {
    let (_fixture, invocation, _) = merge_fixture();
    let (view, _exit_code, cx) = open_tool(invocation, cx);

    let left = cx
        .debug_bounds("external-source-scroll-0")
        .expect("left source pane");
    scroll_wheel(cx, left.center(), px(-400.));
    let (left_line, right_line) = view.read_with(cx, |view, cx| {
        (
            view.merge_pane_center(MergePane::Left, cx).expect("left"),
            view.merge_pane_center(MergePane::Right, cx).expect("right"),
        )
    });
    assert!(left_line > 1., "wheel input should scroll the left source");
    assert!(
        (left_line - right_line).abs() < 1.,
        "the right source should follow the left one ({left_line} vs {right_line})"
    );
    assert_eq!(
        view.read_with(cx, |view, _| view.selected_merge_hunk()),
        0,
        "scrolling must not move the explicit selection"
    );

    let raw = cx
        .debug_bounds("external-result-raw")
        .expect("Raw mode button");
    cx.simulate_click(raw.center(), Modifiers::default());
    settle_visual(cx);
    let source = cx
        .debug_bounds("external-source-scroll-1")
        .expect("right source pane");
    scroll_wheel(cx, source.center(), px(-200.));
    let (right_line, result_line) = view.read_with(cx, |view, cx| {
        (
            view.merge_pane_center(MergePane::Right, cx).expect("right"),
            view.merge_pane_center(MergePane::Result, cx)
                .expect("result"),
        )
    });
    assert!(
        result_line > right_line,
        "the raw result carries both conflict blocks, so it sits below the matching source line ({result_line} vs {right_line})"
    );
}
