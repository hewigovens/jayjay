use std::sync::Arc;

use crate::harness::*;
use gpui::{Focusable, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use jayjay_core::{EdgeType, GraphEdge, GraphEntry};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::ui::context_menu::ContextMenuItem;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test::{LinearFixture, change_by_description, run_jj_in};

/// `graph.changes` mirrors `graph.entries`, so a forged topology has to be written to both.
fn set_parents(vm: &mut RepoViewModel, row: usize, parents: Vec<String>) {
    Arc::make_mut(&mut vm.graph.changes)[row].parents = parents.clone();
    Arc::make_mut(&mut vm.graph.entries)[row].change.parents = parents;
}

#[gpui::test]
fn consecutive_selection_loads_combined_diff_and_topology_gates_batch_menu(
    cx: &mut TestAppContext,
) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![0, 1, 2]);
        assert_eq!(
            vm.selected,
            Some(0),
            "detail should use the comparison target"
        );
        assert_eq!(
            vm.compare
                .as_ref()
                .map(|compare| compare.display.title.as_str()),
            Some("3 Changes Selected")
        );

        let selected = vm.graph.changes[1].clone();
        let menu = view.build_change_menu(&selected, cx);
        assert!(!menu_item(&menu, "Merge 3 selected").enabled);
        assert!(menu_item(&menu, "Squash 3 selected…").enabled);
        assert!(menu_item(&menu, "Parallelize 3 selected").enabled);
        assert!(menu_item(&menu, "Abandon 3 selected…").enabled);

        let destination = vm.graph.changes[3].clone();
        let destination_menu = view.build_change_menu(&destination, cx);
        assert!(menu_item(&destination_menu, "Rebase 3 selected onto this").enabled);
    });
    assert!(cx.debug_bounds("compare-combined-selection").is_some());
    assert!(cx.debug_bounds("compare-reverse").is_none());

    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![2]);
        assert!(vm.compare.is_none());
    });
}

#[gpui::test]
fn adjacent_sibling_selection_keeps_rows_selected_without_loading_a_diff(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let parents = vm.graph.changes[1].parents.clone();
            set_parents(vm, 0, parents);
        });
        view.handle_change_row_click(1, Modifiers::secondary_key(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![0, 1]);
        assert_eq!(vm.selection_without_diff_count(), Some(2));
        assert!(vm.compare.is_none());
    });
}

#[gpui::test]
fn selection_rooted_at_a_merge_keeps_rows_selected_without_loading_a_diff(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let mut parents = vm.graph.changes[1].parents.clone();
            parents.push("second-parent".to_owned());
            set_parents(vm, 1, parents);
        });
        view.handle_change_row_click(1, Modifiers::secondary_key(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![0, 1]);
        assert_eq!(vm.selection_without_diff_count(), Some(2));
        assert!(
            vm.can_squash_selected_changes(),
            "squashing into a merge commit is legal even though its combined diff is not"
        );
        assert!(vm.compare.is_none());
    });
    assert!(
        cx.debug_bounds("detail-multi-selection-no-diff").is_some(),
        "a topology-invalid range should explain why no diff is shown"
    );
    let content = cx
        .debug_bounds("detail-multi-selection-content")
        .expect("constrained multi-selection explanation");
    assert!(
        f32::from(content.size.width) <= 460.,
        "multi-selection guidance should not span the detail pane"
    );
}

#[gpui::test]
fn indirect_visible_edge_disables_related_merge_selection(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);

    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let descendant = vm.graph.changes[0].clone();
            let ancestor = vm.graph.changes[2].clone();
            vm.graph.changes = Arc::new(vec![descendant.clone(), ancestor.clone()]);
            vm.graph.entries = Arc::new(vec![
                GraphEntry {
                    change: descendant,
                    edges: vec![GraphEdge {
                        target: ancestor.commit_id.id.clone(),
                        edge_type: EdgeType::Indirect,
                    }],
                },
                GraphEntry {
                    change: ancestor,
                    edges: Vec::new(),
                },
            ]);
        });
        view.handle_change_row_click(1, Modifiers::secondary_key(), cx);
    });

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![0, 1]);
        assert!(!vm.can_merge_selected_changes());
    });
}

#[gpui::test]
fn nonconsecutive_selection_compares_outermost_changes(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);

    view.update_in(cx, |view, _, cx| {
        view.handle_change_row_click(2, Modifiers::secondary_key(), cx);
    });
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![0, 2]);
        assert_eq!(vm.selection_without_diff_count(), None);
        let compare = vm.compare.as_ref().expect("outermost comparison");
        assert_eq!(
            compare.source_change_id.as_deref(),
            Some(vm.graph.changes[2].change_id.as_str())
        );
        assert_eq!(
            compare.target_change_id.as_deref(),
            Some(vm.graph.changes[0].change_id.as_str())
        );
        assert_eq!(compare.display.title, "Comparing");
    });
    assert!(
        cx.debug_bounds("detail-multi-selection-no-diff").is_none(),
        "non-consecutive selection should show the outermost comparison"
    );
    assert!(cx.debug_bounds("compare-combined-selection").is_none());
    assert!(cx.debug_bounds("compare-reverse").is_some());

    view.update_in(cx, |view, window, cx| {
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_keystrokes("escape");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.view_model().read(cx).selected_change_indices(),
            vec![2]
        );
    });
}

#[gpui::test]
fn squash_batch_action_confirms_then_runs_as_one_mutation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    let action = view.read_with(cx, |view, cx| {
        let selected = view.view_model().read(cx).graph.changes[1].clone();
        menu_item(&view.build_change_menu(&selected, cx), "Squash 3 selected…")
            .action
            .clone()
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);

    let confirm = cx
        .debug_bounds("confirmation-submit")
        .expect("squash confirmation");
    cx.simulate_click(confirm.center(), gpui::Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "batch squash failed: {:?}", vm.error);
        assert!(
            vm.graph
                .changes
                .iter()
                .any(|change| change.description.trim() == "add hello\nadd feature")
        );
        assert!(!vm.has_multiple_change_selection());
    });
}

#[gpui::test]
fn parallelize_batch_action_rewrites_the_selection_into_siblings(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    let action = view.read_with(cx, |view, cx| {
        let selected = view.view_model().read(cx).graph.changes[1].clone();
        let menu = view.build_change_menu(&selected, cx);
        let item = menu_item(&menu, "Parallelize 3 selected");
        assert!(item.enabled, "a linear selection must offer parallelize");
        item.action.clone()
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert_first_three_are_siblings(view.view_model().read(cx))
    });
}

#[gpui::test]
fn parallelize_palette_action_rewrites_the_multi_selection(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    cx.cx.update(|cx| {
        CommandPalette::open(
            fixture.path.display().to_string().into(),
            Some(view.clone()),
            cx,
        );
    });
    let window = cx.cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, &cx.cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("parallelize");
    palette_cx.simulate_keystrokes("enter");
    settle_visual(&mut palette_cx);

    view.read_with(&palette_cx, |view, cx| {
        assert_first_three_are_siblings(view.view_model().read(cx))
    });
}

#[gpui::test]
fn merge_submenu_chooses_first_parent_and_creates_the_ordered_merge(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["parallelize", "@ | @- | @--"]);
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    let (label, row_id, expected) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected_change_indices();
        let changes: Vec<_> = selected
            .iter()
            .map(|&index| &vm.graph.changes[index])
            .collect();
        let menu = view.build_change_menu(changes[0], cx);
        let merge = menu_item(&menu, "Merge 3 selected");
        assert!(merge.enabled);
        let choices = merge.submenu_items().expect("first-parent submenu");
        let choice = choices
            .iter()
            .find(|item| item.label.starts_with(&changes[2].change_id.prefix(8)))
            .expect("third selected change offered as first parent");
        assert!(choice.enabled);
        let expected =
            [changes[2], changes[0], changes[1]].map(|change| change.commit_id.id.clone());
        (
            choice.label.to_string(),
            changes[0].commit_id.id.clone(),
            expected,
        )
    });
    hover_merge_item(&row_id, cx);
    let choice = cx
        .debug_bounds(selector(format!("context-menu-{label}")))
        .expect("first-parent choice");
    cx.simulate_mouse_down(choice.center(), MouseButton::Left, Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "merge failed: {:?}", vm.error);
        let merged = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy)
            .expect("merge working copy");
        assert_eq!(merged.parents, expected);
    });
}

#[gpui::test]
fn hovering_a_disabled_merge_item_keeps_its_submenu_closed(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    let row_id = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        vm.graph.changes[0].commit_id.id.clone()
    });
    hover_merge_item(&row_id, cx);
    assert!(cx.debug_bounds("context-menu-First parent").is_none());
}

fn hover_merge_item(row_id: &str, cx: &mut VisualTestContext) {
    let row = cx
        .debug_bounds(selector(format!("dag-change-{row_id}")))
        .expect("selected row");
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(cx);
    let merge = cx
        .debug_bounds("context-menu-Merge 3 selected")
        .expect("merge menu");
    cx.simulate_mouse_move(merge.center(), MouseButton::Left, Modifiers::default());
    settle_visual(cx);
}

fn assert_first_three_are_siblings(vm: &RepoViewModel) {
    assert!(vm.error.is_none(), "parallelize failed: {:?}", vm.error);
    let initial = &change_by_description(&vm.graph.changes, "initial")
        .commit_id
        .id;
    for subject in ["add hello", "add feature"] {
        assert_eq!(
            change_by_description(&vm.graph.changes, subject).parents,
            std::slice::from_ref(initial),
            "{subject}"
        );
    }
    let working_copy = vm
        .graph
        .changes
        .iter()
        .find(|change| change.is_working_copy)
        .expect("working copy");
    assert_eq!(working_copy.parents, std::slice::from_ref(initial));
    assert!(!vm.has_multiple_change_selection());
}

fn select_first_three(view: &gpui::Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, _, cx| {
        view.handle_change_row_click(
            2,
            Modifiers {
                shift: true,
                ..Default::default()
            },
            cx,
        );
    });
    settle_visual(cx);
}

fn menu_item<'a>(items: &'a [ContextMenuItem], label: &str) -> &'a ContextMenuItem {
    items
        .iter()
        .find(|item| item.label.as_ref() == label)
        .unwrap_or_else(|| {
            let labels: Vec<_> = items.iter().map(|item| item.label.as_ref()).collect();
            panic!("missing {label:?}: {labels:?}")
        })
}
