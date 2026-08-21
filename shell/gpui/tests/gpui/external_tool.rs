use std::cell::Cell;
use std::fs;
use std::rc::Rc;

use crate::harness::{install_test_globals, settle_visual};
use gpui::{
    Entity, Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase,
    VisualTestContext, point, px, size,
};
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

#[gpui::test]
fn merge_tool_accepts_a_hunk_and_saves_the_output(cx: &mut TestAppContext) {
    let fixture = tempfile::tempdir().expect("fixture");
    let left = fixture.path().join("left.rs");
    let base = fixture.path().join("base.rs");
    let right = fixture.path().join("right.rs");
    let output = fixture.path().join("output.rs");
    let content = |offset: i32| {
        let first = (0..8)
            .map(|line| format!("    let first_{line} = {};\n", offset + line))
            .collect::<String>();
        let second = (0..8)
            .map(|line| format!("    let second_{line} = {};\n", offset * 10 + line))
            .collect::<String>();
        format!("fn first() {{\n{first}}}\n\nfn stable() {{}}\n\nfn second() {{\n{second}}}\n")
    };
    let left_content = content(10);
    let base_content = content(0);
    let right_content = content(20);
    fs::write(&left, &left_content).expect("left");
    fs::write(&base, &base_content).expect("base");
    fs::write(&right, &right_content).expect("right");
    fs::write(&output, "").expect("output");
    let invocation = ExternalToolInvocation::Merge {
        left: left.to_string_lossy().into_owned(),
        base: base.to_string_lossy().into_owned(),
        right: right.to_string_lossy().into_owned(),
        output: output.to_string_lossy().into_owned(),
        path: "src/value.rs".to_owned(),
        marker_length: 7,
    };
    let (_view, exit_code, cx) = open_tool(invocation, cx);

    let use_first_right = cx
        .debug_bounds("external-hunk-0-Accept Right")
        .expect("first Accept Right hunk action");
    cx.simulate_click(use_first_right.center(), Modifiers::default());
    settle_visual(cx);
    let list = cx.debug_bounds("external-hunks-scroll").expect("hunk list");
    let second_before = cx.debug_bounds("merge-hunk-1").expect("second merge hunk");
    cx.simulate_event(ScrollWheelEvent {
        position: list.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        modifiers: Default::default(),
        touch_phase: TouchPhase::Moved,
    });
    settle_visual(cx);
    let second_after = cx
        .debug_bounds("merge-hunk-1")
        .expect("scrolled second merge hunk");
    assert!(
        second_after.origin.y < second_before.origin.y,
        "wheel input should scroll the second hunk toward the viewport"
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
