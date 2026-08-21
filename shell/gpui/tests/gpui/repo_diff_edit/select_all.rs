use std::fs;

use gpui::TestAppContext;
use jayjay_core::{DiffEditDestination, HunkType};
use jayjay_gpui::repo::window::DiffEditCheckboxState;
use jj_test::{LinearFixture, run_jj_in};

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn select_all_waits_for_uncached_files_and_selects_every_file(cx: &mut TestAppContext) {
    let fixture = two_file_working_copy_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    let paths = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let paths = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .map(|hunk| hunk.path.clone())
            .collect::<Vec<_>>();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_all(cx);
        assert!(view.diff_edit_selecting_all());
        paths
    });

    settle_visual(cx);
    assert!(!view.read_with(cx, |view, _| view.diff_edit_selecting_all()));
    view.update_in(cx, |view, _, _| {
        for path in paths {
            assert_eq!(
                view.diff_edit_file_state(&path),
                DiffEditCheckboxState::All,
                "{path} must not be silently skipped"
            );
        }
    });
}

#[gpui::test]
fn divergent_change_preloads_uncached_files_for_select_all(cx: &mut TestAppContext) {
    let fixture = two_file_working_copy_fixture();
    run_jj_in(&fixture.path, &["describe", "-m", "side one"]);
    run_jj_in(
        &fixture.path,
        &["--at-operation", "@-", "describe", "-m", "side two"],
    );
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    let (paths, uncached_path) = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected_change().expect("selected change");
        assert!(selected.is_divergent, "fixture change must be divergent");
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let hunks = vm.files.as_ref().expect("files loaded");
        let paths = hunks
            .iter()
            .map(|hunk| hunk.path.clone())
            .collect::<Vec<_>>();
        let uncached_path = paths
            .iter()
            .find(|path| **path != selected_path)
            .expect("second file")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_all(cx);
        (paths, uncached_path)
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        assert!(!view.diff_edit_selecting_all());
        let uncached = view
            .view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path == uncached_path)
            .expect("previously uncached file")
            .clone();
        assert!(view.diff_edit_file_supported(&uncached));
        for path in paths {
            assert_eq!(
                view.diff_edit_file_state(&path),
                DiffEditCheckboxState::All,
                "{path} must be selected"
            );
        }
    });
}

#[gpui::test]
fn select_all_finishes_when_an_uncached_preload_fails(cx: &mut TestAppContext) {
    let fixture = two_file_working_copy_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        let selected_ix = view
            .view_model()
            .read(cx)
            .selected_file_ix
            .expect("selected file");
        view.view_model().update(cx, |vm, _| {
            let selected_path = vm
                .files
                .as_ref()
                .and_then(|files| files.get(selected_ix))
                .expect("selected hunk")
                .path
                .clone();
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
            let files = std::sync::Arc::make_mut(vm.files.as_mut().expect("files loaded"));
            let failing = files
                .iter_mut()
                .enumerate()
                .find(|(ix, _)| *ix != selected_ix)
                .expect("uncached hunk")
                .1;
            failing.path = "missing-after-summary.txt".to_owned();
        });
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_all(cx);
        assert!(view.diff_edit_selecting_all());
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        assert!(!view.diff_edit_selecting_all());
        assert!(view.diff_edit_has_known_unsupported(cx));
    });
}

#[gpui::test]
fn renamed_file_is_excluded_while_supported_edits_apply(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::create_dir(fixture.path.join("moved")).unwrap();
    fs::rename(
        fixture.path.join("README.md"),
        fixture.path.join("moved/README.md"),
    )
    .unwrap();
    let renamed = "# Sample project\nrenamed edit\n";
    fs::write(fixture.path.join("moved/README.md"), renamed).unwrap();
    fs::write(fixture.path.join("feature.txt"), "feature\nedited\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let renamed_hunk = vm
            .files
            .as_ref()
            .unwrap()
            .iter()
            .find(|hunk| hunk.path == "moved/README.md")
            .expect("renamed hunk");
        assert_eq!(renamed_hunk.hunk_type, HunkType::Renamed);
        assert!(!view.diff_edit_file_supported(renamed_hunk));
        view.enter_diff_edit(cx);
        view.toggle_diff_edit_file("moved/README.md", cx);
        assert!(view.diff_edit_selected("moved/README.md").is_empty());
        view.toggle_diff_edit_all(cx);
    });
    settle_visual(cx);

    view.update_in(cx, |view, _, cx| {
        assert!(!view.diff_edit_selecting_all());
        assert!(view.diff_edit_selected("moved/README.md").is_empty());
        assert_eq!(
            view.diff_edit_file_state("feature.txt"),
            DiffEditCheckboxState::All
        );
        view.start_diff_edit_apply(DiffEditDestination::RemoveFromSource, cx);
    });
    settle_visual(cx);
    settle_visual(cx);

    assert_eq!(
        fs::read_to_string(fixture.path.join("feature.txt")).unwrap(),
        "feature\nedited\n"
    );
    assert!(!fixture.path.join("README.md").exists());
    assert_eq!(
        fs::read_to_string(fixture.path.join("moved/README.md")).unwrap(),
        renamed
    );
}
