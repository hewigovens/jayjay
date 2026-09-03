use std::sync::Arc;

use crate::harness::*;
use gpui::{Focusable, Modifiers, TestAppContext, VisualTestContext};
use jayjay_core::{EdgeType, GraphEdge, GraphEntry};
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::ui::context_menu::ContextMenuItem;
use jj_test::LinearFixture;

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
            let changes = Arc::make_mut(&mut vm.graph.changes);
            changes[0].parents = changes[1].parents.clone();
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
            Arc::make_mut(&mut vm.graph.changes)[1]
                .parents
                .push("second-parent".to_owned());
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
