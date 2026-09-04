use crate::harness::*;
use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext, WindowHandle};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::revset;
use jayjay_gpui::ui::context_menu::ContextAction;
use jayjay_gpui::windows::evolog::EvologView;
use jj_test::{LinearFixture, run_jj_in};

fn open_evolog(fixture: &LinearFixture, cx: &mut TestAppContext) -> VisualTestContext {
    let (view, repo_cx) = open_fixture(fixture, cx);
    show_evolog(&view, repo_cx)
}

fn show_evolog(view: &Entity<RepoWindow>, repo_cx: &mut VisualTestContext) -> VisualTestContext {
    let rev = view.read_with(repo_cx, |view, cx| {
        revset::change_revision(
            view.view_model()
                .read(cx)
                .selected_change()
                .expect("working copy"),
        )
    });
    view.update_in(repo_cx, |view, _, cx| {
        view.dispatch_context_action(ContextAction::OpenEvologFor(rev.into()), cx);
    });
    settle_visual(repo_cx);
    let window = repo_cx
        .cx
        .windows()
        .into_iter()
        .find(|window| window.downcast::<EvologView>().is_some())
        .expect("evolog window");
    let mut evolog_cx = VisualTestContext::from_window(window, &repo_cx.cx);
    settle_visual(&mut evolog_cx);
    evolog_cx
}

fn evolog_window(cx: &VisualTestContext) -> WindowHandle<EvologView> {
    cx.cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<EvologView>())
        .expect("evolog window")
}

#[gpui::test]
fn evolog_uses_the_global_font_size(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let mut evolog_cx = open_evolog(&fixture, cx);
    let before = rendered_height(&mut evolog_cx, "evolog-title");

    zoom_to_max(&mut evolog_cx);

    let after = rendered_height(&mut evolog_cx, "evolog-title");
    assert!(
        after > before,
        "global zoom should increase the Evolog title from {before:?}, got {after:?}"
    );
}

fn select_version(evolog: &WindowHandle<EvologView>, cx: &mut VisualTestContext, index: usize) {
    evolog
        .update(&mut cx.cx, |view, _, cx| {
            view.select_version(index, Modifiers::default(), cx);
        })
        .expect("select version");
    settle_visual(cx);
}

fn snapshot_working_copy(fixture: &LinearFixture, contents: &str) {
    std::fs::write(fixture.path.join("wip1.txt"), contents).expect("write working copy");
    run_jj_in(&fixture.path, &["st"]);
}

fn snapshot_image(fixture: &LinearFixture, source: &str) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(source);
    std::fs::copy(source, fixture.path.join("preview.png")).expect("copy image");
    run_jj_in(&fixture.path, &["st"]);
}

const COLLAPSED_RUN: &str = "evolog-snapshot-run-1-11";

fn click_hide_toggle(cx: &mut VisualTestContext) {
    let toggle = cx
        .debug_bounds("evolog-hide-snapshots")
        .expect("hide snapshots toggle");
    cx.simulate_click(toggle.center(), Modifiers::default());
    settle_visual(cx);
}

#[gpui::test]
fn evolog_hides_consecutive_snapshots_until_toggled_or_expanded(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["describe", "-m", "described"]);
    for index in 0..12 {
        snapshot_working_copy(&fixture, &format!("snap {index}\n"));
    }

    let mut evolog_cx = open_evolog(&fixture, cx);
    assert!(
        evolog_cx.debug_bounds(COLLAPSED_RUN).is_some(),
        "collapsed snapshot run"
    );

    click_hide_toggle(&mut evolog_cx);
    assert!(
        evolog_cx.debug_bounds(COLLAPSED_RUN).is_none(),
        "turning hide off should show every snapshot row"
    );

    click_hide_toggle(&mut evolog_cx);
    let expand_bounds = evolog_cx
        .debug_bounds(selector(format!("{COLLAPSED_RUN}-label")))
        .expect("collapsed run label");
    evolog_cx.simulate_click(expand_bounds.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    assert!(
        evolog_cx.debug_bounds(COLLAPSED_RUN).is_none(),
        "clicking a collapsed run should expand it"
    );

    click_hide_toggle(&mut evolog_cx);
    click_hide_toggle(&mut evolog_cx);
    assert!(
        evolog_cx.debug_bounds(COLLAPSED_RUN).is_some(),
        "hiding again should collapse the expanded run"
    );
}

#[gpui::test]
fn evolog_modifier_selection_diffs_at_most_two_versions(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["describe", "-m", "described"]);
    for index in 0..4 {
        snapshot_working_copy(&fixture, &format!("version {index}\n"));
    }

    let mut evolog_cx = open_evolog(&fixture, cx);
    click_hide_toggle(&mut evolog_cx);
    let evolog = evolog_window(&evolog_cx);

    evolog
        .update(&mut evolog_cx.cx, |view, _, cx| {
            view.select_version(2, Modifiers::default(), cx);
            view.select_version(1, secondary_modifiers(), cx);
        })
        .expect("select versions");
    settle_visual(&mut evolog_cx);
    evolog
        .read_with(&evolog_cx.cx, |view, _| {
            assert_eq!(view.selected_version_indices(), vec![1, 2]);
            assert_eq!(view.selected_diff_path(), Some("wip1.txt"));
        })
        .expect("read selected versions");
    assert!(
        evolog_cx.debug_bounds("evolog-compare-banner").is_some(),
        "selected endpoints should be identified above the diff"
    );
    let before_reverse = evolog
        .read_with(&evolog_cx.cx, |view, _| view.selected_endpoints())
        .expect("read endpoints")
        .expect("selected endpoints");
    let reverse = evolog_cx
        .debug_bounds("evolog-compare-reverse")
        .expect("reverse comparison button");
    evolog_cx.simulate_click(reverse.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    evolog
        .read_with(&evolog_cx.cx, |view, _| {
            assert_eq!(
                view.selected_endpoints(),
                Some((before_reverse.1.clone(), before_reverse.0.clone()))
            );
            assert_eq!(view.selected_diff_path(), Some("wip1.txt"));
        })
        .expect("read reversed endpoints");

    let row = evolog_cx
        .debug_bounds(selector(format!("evolog-row-{}", before_reverse.0)))
        .expect("version row");
    evolog_cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(&mut evolog_cx);
    let restore = evolog_cx
        .debug_bounds("evolog-context-copy-restore")
        .expect("restore context menu item");
    evolog_cx.simulate_click(restore.center(), Modifiers::default());
    assert_eq!(
        evolog_cx
            .cx
            .read_from_clipboard()
            .and_then(|item| item.text()),
        Some(format!("jj restore --from {} --into @", before_reverse.0))
    );

    let version_id = evolog_cx
        .debug_bounds(selector(format!("commit-{}", before_reverse.0)))
        .expect("version id");
    evolog_cx.simulate_click(version_id.center(), Modifiers::default());
    settle_visual(&mut evolog_cx);
    evolog
        .read_with(&evolog_cx.cx, |view, _| {
            assert_eq!(view.selected_version_indices(), vec![2]);
        })
        .expect("select version through its id");

    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    evolog
        .update(&mut evolog_cx.cx, |view, _, cx| {
            view.select_version(3, shift, cx);
        })
        .expect("extend selection");
    settle_visual(&mut evolog_cx);
    evolog
        .read_with(&evolog_cx.cx, |view, _| {
            assert_eq!(view.selected_version_indices(), vec![2, 3]);
        })
        .expect("read endpoint selection");
}

#[gpui::test]
fn evolog_renders_image_interdiff_preview(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    snapshot_image(&fixture, "docs/apple-touch-icon.png");
    snapshot_image(&fixture, "docs/imgs/home.png");

    let mut evolog_cx = open_evolog(&fixture, cx);
    let evolog = evolog_window(&evolog_cx);
    evolog
        .update(&mut evolog_cx.cx, |view, _, cx| {
            view.select_version(1, Modifiers::default(), cx);
        })
        .expect("select image version");
    settle_visual(&mut evolog_cx);

    assert!(
        evolog_cx.debug_bounds("image-preview-pane").is_some(),
        "image interdiff should use the image renderer"
    );
}

#[gpui::test]
fn evolog_pane_widths_survive_version_switch_and_reopen(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["describe", "-m", "described"]);
    for index in 0..3 {
        snapshot_working_copy(&fixture, &format!("version {index}\n"));
    }
    let (repo_view, repo_cx) = open_fixture(&fixture, cx);
    let mut evolog_cx = show_evolog(&repo_view, repo_cx);
    click_hide_toggle(&mut evolog_cx);

    let entry_initial = pane_width(&mut evolog_cx, "evolog-entry-list");
    drag_handle(&mut evolog_cx, "evolog-entry-list-resize-handle", 60.);
    let entry_resized = pane_width(&mut evolog_cx, "evolog-entry-list");
    assert!(
        entry_resized > entry_initial + 50.,
        "drag should widen the entry list: {entry_initial} -> {entry_resized}"
    );

    let evolog = evolog_window(&evolog_cx);
    select_version(&evolog, &mut evolog_cx, 1);
    let file_initial = pane_width(&mut evolog_cx, "evolog-file-list");
    drag_handle(&mut evolog_cx, "evolog-file-list-resize-handle", 40.);
    let file_resized = pane_width(&mut evolog_cx, "evolog-file-list");
    assert!(
        file_resized > file_initial + 30.,
        "drag should widen the file list: {file_initial} -> {file_resized}"
    );

    select_version(&evolog, &mut evolog_cx, 2);
    assert_eq!(
        pane_width(&mut evolog_cx, "evolog-entry-list"),
        entry_resized
    );
    assert_eq!(pane_width(&mut evolog_cx, "evolog-file-list"), file_resized);

    evolog
        .update(&mut evolog_cx.cx, |_, window, _| window.remove_window())
        .expect("close evolog");
    settle_visual(repo_cx);
    let mut reopened_cx = show_evolog(&repo_view, repo_cx);
    click_hide_toggle(&mut reopened_cx);
    assert_eq!(
        pane_width(&mut reopened_cx, "evolog-entry-list"),
        entry_resized
    );
    select_version(&evolog_window(&reopened_cx), &mut reopened_cx, 1);
    assert_eq!(
        pane_width(&mut reopened_cx, "evolog-file-list"),
        entry_resized,
        "the file list should start from the shared pane width, not its own last drag"
    );
}

fn secondary_modifiers() -> Modifiers {
    #[cfg(target_os = "macos")]
    {
        Modifiers {
            platform: true,
            ..Default::default()
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Modifiers {
            control: true,
            ..Default::default()
        }
    }
}
