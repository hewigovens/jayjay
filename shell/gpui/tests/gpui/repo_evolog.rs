use crate::harness::*;
use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext};
use jayjay_gpui::repo::revset;
use jayjay_gpui::ui::context_menu::ContextAction;
use jayjay_gpui::windows::evolog::EvologView;
use jj_test::{LinearFixture, run_jj_in};

fn open_evolog(fixture: &LinearFixture, cx: &mut TestAppContext) -> VisualTestContext {
    let (view, repo_cx) = open_fixture(fixture, cx);
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
    let evolog = evolog_cx
        .cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<EvologView>())
        .expect("evolog window");

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
    let evolog = evolog_cx
        .cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<EvologView>())
        .expect("evolog window");
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
