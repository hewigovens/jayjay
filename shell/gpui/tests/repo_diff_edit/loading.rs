use std::fs;

use gpui::TestAppContext;
use jayjay_core::diff::compute_file_diff;
use jayjay_gpui::repo::view_model::LoadedDiff;
use jayjay_gpui::repo::window::DiffEditCheckboxState;
use jj_test::{LinearFixture, run_jj_in};

use super::fixtures::*;
use super::harness::*;

#[gpui::test]
fn uncached_file_is_hidden_until_entry_preload_finishes(cx: &mut TestAppContext) {
    let fixture = two_file_working_copy_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    let uncached_path = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let uncached = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path != selected_path)
            .expect("second file")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
        });
        view.enter_diff_edit(cx);
        assert!(!view.diff_edit_has_known_unsupported(cx));
        assert!(!view.diff_edit_file_supported(&uncached));
        uncached.path
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .files
            .as_ref()
            .unwrap()
            .iter()
            .find(|hunk| hunk.path == uncached_path)
            .unwrap()
            .clone();
        assert!(view.diff_edit_file_supported(&hunk));
        view.toggle_diff_edit_file(&uncached_path, cx);
        assert_eq!(
            view.diff_edit_file_state(&uncached_path),
            DiffEditCheckboxState::All
        );
    });
}

#[gpui::test]
fn same_path_cache_entry_from_another_revision_is_ignored(cx: &mut TestAppContext) {
    let fixture = two_file_working_copy_fixture();
    let (view, cx) = open_fixture(&fixture, cx);
    let stale_path = view.update_in(cx, |view, _, cx| {
        let vm = view.view_model().read(cx);
        let selected_path = vm.selected_hunk().expect("selected file").path.clone();
        let stale_hunk = vm
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path != selected_path)
            .expect("second file")
            .clone();
        view.view_model().update(cx, |vm, _| {
            vm.diff_cache
                .retain(|_, loaded| loaded.diff.path == selected_path);
            vm.diff_cache.insert(
                "another-revision".into(),
                LoadedDiff {
                    diff: std::sync::Arc::new(compute_file_diff(
                        &stale_hunk.path,
                        "old stale\n",
                        "new stale\n",
                        false,
                    )),
                    projection: None,
                    svg_preview: None,
                    markdown_preview: None,
                    old_content: Some("old stale\n".into()),
                    new_content: Some("new stale\n".into()),
                    supports_file_editor: true,
                },
            );
        });
        view.enter_diff_edit(cx);
        assert!(
            !view.diff_edit_file_supported(&stale_hunk),
            "a same-path entry with the wrong cache key must not be consumed"
        );
        stale_hunk.path
    });

    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        let hunk = view
            .view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .find(|hunk| hunk.path == stale_path)
            .expect("stale-path hunk")
            .clone();
        assert!(view.diff_edit_file_supported(&hunk));
    });
}

#[gpui::test]
fn unsupported_preview_replaces_cached_placeholder_when_cache_grows(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("analysis.ipynb"),
        "{\n \"cells\": [\n  {\"cell_type\": \"markdown\", \"metadata\": {}, \"source\": [\"# Title\"]}\n ],\n \"metadata\": {},\n \"nbformat\": 4,\n \"nbformat_minor\": 5\n}\n",
    )
    .unwrap();
    fs::write(fixture.path.join("plain.txt"), "keep\n").unwrap();
    run_jj_in(&fixture.path, &["st"]);

    let (view, cx) = open_fixture(&fixture, cx);
    select_file(&view, "plain.txt", cx);
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| vm.diff_cache.clear());
        view.enter_diff_edit(cx);
        assert_eq!(
            view.diff_edit_preview_line_count("analysis.ipynb", cx),
            0,
            "the row model must begin with a placeholder"
        );
    });

    settle_visual(cx);
    let preview_lines = view.update_in(cx, |view, _, cx| {
        view.diff_edit_preview_line_count("analysis.ipynb", cx)
    });
    assert!(
        preview_lines > 0,
        "the preload's arrival must replace the cached placeholder"
    );
}
