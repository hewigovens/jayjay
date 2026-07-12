use gpui::TestAppContext;
use jayjay_core::diff::{DiffSpanStyle, compute_file_diff};
use jayjay_gpui::repo::view_model::LoadedDiff;
use jayjay_gpui::repo::window::DiffEditCheckboxState;

use super::common::*;

#[gpui::test]
fn line_group_file_and_select_all_update_full_diff_selection(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let (path, changed) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let diff = vm.current_diff.as_ref().unwrap();
        let changed = diff
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| matches!(line.style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
            .map(|(ix, _)| ix)
            .collect::<Vec<_>>();
        (vm.selected_hunk().unwrap().path.clone(), changed)
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert_eq!(view.diff_edit_selected(&path).len(), 1);
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert!(view.diff_edit_selected(&path).is_empty());
        view.select_diff_edit_display_group(&path, changed[0] as u32 + 1, cx);
        assert!(!view.diff_edit_selected(&path).is_empty());
        view.toggle_diff_edit_display_line(&path, changed[0] as u32 + 1, cx);
        assert_eq!(
            view.diff_edit_file_state(&path),
            DiffEditCheckboxState::Some
        );
        view.toggle_diff_edit_file(&path, cx);
        assert_eq!(view.diff_edit_file_state(&path), DiffEditCheckboxState::All);
        view.toggle_diff_edit_file(&path, cx);
        assert_eq!(
            view.diff_edit_file_state(&path),
            DiffEditCheckboxState::None
        );
        view.toggle_diff_edit_all(cx);
        assert_eq!(view.diff_edit_file_state(&path), DiffEditCheckboxState::All);
    });
}

#[gpui::test]
fn collapsed_display_line_maps_to_full_diff_index(cx: &mut TestAppContext) {
    let (_fixture, view, cx) = open_changed_repo(cx);
    let old = (1..=40)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    let new = old.replace("line 20\n", "changed 20\n");
    let display = std::sync::Arc::new(compute_file_diff("README.md", &old, &new, false));
    let changed_display_ix = display
        .lines
        .iter()
        .position(|line| line.style == DiffSpanStyle::Removed)
        .unwrap();
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let cache_key = vm
                .diff_cache
                .iter()
                .find(|(_, loaded)| loaded.diff.path == "README.md")
                .map(|(key, _)| key.clone())
                .expect("selected diff cache key");
            vm.current_diff = Some(display.clone());
            vm.current_diff_old_content = Some(old.clone().into());
            vm.current_diff_new_content = Some(new.clone().into());
            vm.diff_cache.clear();
            vm.diff_cache.insert(
                cache_key,
                LoadedDiff {
                    diff: display,
                    projection: None,
                    svg_preview: None,
                    markdown_preview: None,
                    old_content: Some(old.into()),
                    new_content: Some(new.into()),
                },
            );
        });
        view.enter_diff_edit(cx);
    });
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line("README.md", changed_display_ix as u32 + 1, cx);
        let selected = view.diff_edit_selected("README.md");
        assert_eq!(selected.len(), 1);
        assert!(selected.first().copied().unwrap() > changed_display_ix as u32 + 1);
    });
}

#[gpui::test]
fn group_selection_is_idempotent(cx: &mut TestAppContext) {
    let fixture = separated_edits_fixture(false);
    let (view, cx) = open_fixture(&fixture, cx);
    select_file_by_path(&view, cx, "edit.txt");
    let display_line = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .current_diff
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == "selected two")
            .unwrap() as u32
            + 1
    });
    view.update_in(cx, |view, _, cx| {
        view.enter_diff_edit(cx);
        view.select_diff_edit_display_group("edit.txt", display_line, cx);
    });
    let first = view.read_with(cx, |view, _| view.diff_edit_selected("edit.txt"));
    view.update_in(cx, |view, _, cx| {
        view.select_diff_edit_display_group("edit.txt", display_line, cx)
    });
    assert_eq!(
        view.read_with(cx, |view, _| view.diff_edit_selected("edit.txt")),
        first
    );
}
