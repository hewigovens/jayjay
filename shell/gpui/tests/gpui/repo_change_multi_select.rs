use std::sync::Arc;

use crate::harness::*;
use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use jayjay_core::{ChangeInfo, EdgeType, GraphEdge, GraphEntry};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::repo::view_model::RepoViewModel;
use jayjay_gpui::ui::context_menu::ContextMenuItem;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test::{LinearFixture, change_by_description, run_jj_in};

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
        assert_eq!(vm.selected, Some(0));
        assert_eq!(
            vm.compare
                .as_ref()
                .map(|compare| compare.display.title.as_str()),
            Some("3 Changes Selected")
        );

        let menu = view.build_change_menu(&vm.graph.changes[1], cx);
        assert!(!menu_item(&menu, "Merge 3 selected").enabled);
        for label in [
            "Squash 3 selected…",
            "Parallelize 3 selected",
            "Abandon 3 selected…",
        ] {
            assert!(menu_item(&menu, label).enabled, "{label}");
        }
        let destination_menu = view.build_change_menu(&vm.graph.changes[3], cx);
        assert!(menu_item(&destination_menu, "Rebase 3 selected onto this").enabled);
    });
    assert!(cx.debug_bounds("compare-combined-selection").is_some());
    assert!(cx.debug_bounds("compare-reverse").is_none());

    focus_view(&view, cx);
    keys(cx, "escape");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert_eq!(vm.selected_change_indices(), vec![2]);
        assert!(vm.compare.is_none());
    });
}

#[gpui::test]
fn sibling_or_merge_rooted_selection_keeps_rows_selected_without_a_diff(cx: &mut TestAppContext) {
    let make_siblings = |vm: &mut RepoViewModel| {
        let parents = vm.graph.changes[1].parents.clone();
        edit_change(vm, 0, |change| change.parents = parents.clone());
    };
    let make_merge = |vm: &mut RepoViewModel| {
        edit_change(vm, 1, |change| {
            change.parents.push("second-parent".to_owned())
        });
    };

    for (forge, is_merge) in [
        (&make_siblings as &dyn Fn(&mut RepoViewModel), false),
        (&make_merge, true),
    ] {
        let fixture = LinearFixture::build();
        let (view, cx) = open_fixture(&fixture, cx);
        view.update_in(cx, |view, _, cx| {
            view.view_model().update(cx, |vm, _| forge(vm));
            view.handle_change_row_click(1, Modifiers::secondary_key(), cx);
        });
        settle_visual(cx);

        view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            assert_eq!(vm.selected_change_indices(), vec![0, 1]);
            assert_eq!(vm.selection_without_diff_count(), Some(2));
            assert!(vm.compare.is_none());
            if is_merge {
                assert!(vm.can_squash_selected_changes());
            }
        });
        if is_merge {
            assert!(cx.debug_bounds("detail-multi-selection-no-diff").is_some());
            let content = cx
                .debug_bounds("detail-multi-selection-content")
                .expect("multi-selection explanation");
            assert!(f32::from(content.size.width) <= 460.);
        }
    }
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
    assert!(cx.debug_bounds("detail-multi-selection-no-diff").is_none());
    assert!(cx.debug_bounds("compare-combined-selection").is_none());
    assert!(cx.debug_bounds("compare-reverse").is_some());

    focus_view(&view, cx);
    keys(cx, "escape");
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

    dispatch_menu_item(&view, cx, 1, "Squash 3 selected…");
    let confirm = cx
        .debug_bounds("confirmation-submit")
        .expect("squash confirmation");
    cx.simulate_click(confirm.center(), Modifiers::default());
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
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
fn parallelize_from_menu_or_palette_rewrites_the_selection_into_siblings(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);
    dispatch_menu_item(&view, cx, 1, "Parallelize 3 selected");
    view.read_with(cx, |view, cx| {
        assert_first_three_are_siblings(view.view_model().read(cx))
    });

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
        let changes = &view.view_model().read(cx).graph.changes;
        let menu = view.build_change_menu(&changes[0], cx);
        let merge = menu_item(&menu, "Merge 3 selected");
        assert!(merge.enabled);
        let choice = merge
            .submenu_items()
            .expect("first-parent submenu")
            .iter()
            .find(|item| item.label.starts_with(&changes[2].change_id.prefix(8)))
            .expect("third selected change offered as first parent");
        assert!(choice.enabled);
        let expected = [2, 0, 1].map(|index| changes[index].commit_id.id.clone());
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
        assert!(vm.error.is_none(), "{:?}", vm.error);
        let merged = vm
            .graph
            .changes
            .iter()
            .find(|change| change.is_working_copy);
        assert_eq!(merged.expect("merge working copy").parents, expected);
    });
}

#[gpui::test]
fn hovering_a_disabled_merge_item_keeps_its_submenu_closed(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    select_first_three(&view, cx);

    let row_id = view.read_with(cx, |view, cx| {
        view.view_model().read(cx).graph.changes[0]
            .commit_id
            .id
            .clone()
    });
    hover_merge_item(&row_id, cx);
    assert!(!menu_shows(cx, "First parent"));
}

#[gpui::test]
fn compare_with_selected_compares_the_selected_change_to_the_menu_target(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);
    view.update_in(cx, |view, _, cx| view.select_change(1, cx));
    settle_visual(cx);
    let [selected, target] = changes(&view, cx, [1, 2]);

    dispatch_menu_item(&view, cx, 2, "Compare with selected");

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let compare = vm.compare.as_ref().expect("comparison");
        assert_eq!(compare.from_rev, selected.selection_revision());
        assert_eq!(compare.to_rev, target.selection_revision());
        assert_eq!(compare.display.from, selected.label());
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert!(vm.files.as_ref().is_some_and(|files| !files.is_empty()));
    });
}

#[gpui::test]
fn divergent_compare_uses_commit_ids_for_one_or_several_siblings(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_fixture(&fixture, cx);

    make_divergent(&view, cx, &[1, 2]);
    let [target, sibling] = changes(&view, cx, [1, 2]);
    dispatch_menu_item(&view, cx, 1, "Compare Divergent Version");
    assert_compare(&view, cx, &sibling.commit_id.id, &target.commit_id.id);

    make_divergent(&view, cx, &[1, 2, 3]);
    let [second, third] = changes(&view, cx, [2, 3]);
    let action = view.read_with(cx, |view, cx| {
        let menu = view.build_change_menu(&view.view_model().read(cx).graph.changes[1], cx);
        let choices = menu_item(&menu, "Compare Divergent Versions")
            .submenu_items()
            .expect("sibling submenu");
        assert_eq!(choices.len(), 2);
        assert!(choices[0].label.starts_with(&second.commit_id.prefix(8)));
        assert!(choices[1].label.starts_with(&third.commit_id.prefix(8)));
        choices[1].action.clone()
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);
    assert_compare(&view, cx, &third.commit_id.id, &target.commit_id.id);
}

// `graph.changes` mirrors `graph.entries`, so a forged change has to be written to both.
fn edit_change(vm: &mut RepoViewModel, row: usize, edit: impl Fn(&mut ChangeInfo)) {
    edit(&mut Arc::make_mut(&mut vm.graph.changes)[row]);
    edit(&mut Arc::make_mut(&mut vm.graph.entries)[row].change);
}

fn make_divergent(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, rows: &[usize]) {
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let shared = vm.graph.changes[rows[0]].change_id.clone();
            for &row in rows {
                edit_change(vm, row, |change| {
                    change.change_id = shared.clone();
                    change.is_divergent = true;
                });
            }
        });
    });
}

fn changes<const N: usize>(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    rows: [usize; N],
) -> [ChangeInfo; N] {
    view.read_with(cx, |view, cx| {
        let changes = &view.view_model().read(cx).graph.changes;
        rows.map(|row| changes[row].clone())
    })
}

fn dispatch_menu_item(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    row: usize,
    label: &str,
) {
    let action = view.read_with(cx, |view, cx| {
        let change = &view.view_model().read(cx).graph.changes[row];
        let menu = view.build_change_menu(change, cx);
        let item = menu_item(&menu, label);
        assert!(item.enabled, "{label} is disabled");
        item.action.clone()
    });
    view.update_in(cx, |view, _, cx| view.dispatch_context_action(action, cx));
    settle_visual(cx);
}

fn assert_compare(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, from: &str, to: &str) {
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let compare = vm.compare.as_ref().expect("comparison");
        assert_eq!(
            (compare.from_rev.as_str(), compare.to_rev.as_str()),
            (from, to)
        );
        assert!(vm.error.is_none(), "{:?}", vm.error);
    });
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
    assert!(vm.error.is_none(), "{:?}", vm.error);
    let initial = &change_by_description(&vm.graph.changes, "initial")
        .commit_id
        .id;
    let working_copy = vm
        .graph
        .changes
        .iter()
        .find(|change| change.is_working_copy);
    for change in [
        change_by_description(&vm.graph.changes, "add hello"),
        change_by_description(&vm.graph.changes, "add feature"),
        working_copy.expect("working copy"),
    ] {
        assert_eq!(
            change.parents,
            std::slice::from_ref(initial),
            "{}",
            change.description
        );
    }
    assert!(!vm.has_multiple_change_selection());
}

fn select_first_three(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, _, cx| {
        view.handle_change_row_click(2, Modifiers::shift(), cx);
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
