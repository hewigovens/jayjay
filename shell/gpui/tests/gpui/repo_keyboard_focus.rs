use crate::harness::*;
use gpui::{Entity, Modifiers, TestAppContext, VisualContext, VisualTestContext, px, size};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::{ActivePane, FocusStop, RepoWindow};
use jj_test::LinearFixture;

fn tree_mode(cx: &mut VisualTestContext) -> bool {
    cx.cx.read(|cx| config::current(cx).diff.tree_file_list)
}

#[gpui::test]
fn horizontal_keys_stay_in_the_change_list_and_return_from_files(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    let selected = view.read_with(cx, |view, cx| view.view_model().read(cx).selected);
    view.update(cx, |view, cx| {
        view.view_model()
            .update(cx, |vm, _| vm.selected_file_ix = None);
    });

    for (left, right) in [("h", "l"), ("left", "right")] {
        keys(cx, right);
        view.read_with(cx, |view, cx| {
            assert_eq!(view.active_pane(), ActivePane::Sidebar);
            assert_eq!(view.view_model().read(cx).selected, selected);
        });
        keys(cx, "tab");
        view.read_with(cx, |view, cx| {
            assert_eq!(view.active_pane(), ActivePane::FileColumn);
            assert_eq!(view.view_model().read(cx).selected_file_ix, Some(0));
        });
        keys(cx, left);
        view.read_with(cx, |view, _| {
            assert_eq!(view.active_pane(), ActivePane::Sidebar)
        });
    }

    cx.simulate_keystrokes("shift-l alt-l ctrl-l");
    assert_eq!(
        view.read_with(cx, |view, _| view.active_pane()),
        ActivePane::Sidebar
    );
    view.update(cx, |view, cx| view.toggle_sidebar(cx));
    settle_slide(cx);
    cx.simulate_keystrokes("h left");
    assert_eq!(
        view.read_with(cx, |view, _| view.active_pane()),
        ActivePane::FileColumn
    );
}

#[gpui::test]
fn tab_enters_the_file_list_then_lands_on_its_toggles(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    view.update(cx, |view, cx| {
        view.view_model().update(cx, |vm, _| {
            vm.selected_file_ix = None;
        });
    });

    keys(cx, "tab");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.view_model().read(cx).selected_file_ix, Some(0));
    });

    keys(cx, "tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::TreeToggle));
    });

    let was_tree_mode = tree_mode(cx);
    keys(cx, "space");
    assert_ne!(tree_mode(cx), was_tree_mode);

    keys(cx, "escape");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    keys(cx, "shift-tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.active_pane(), ActivePane::Sidebar);
        assert_eq!(view.focused_control(), None);
    });
}

#[gpui::test]
fn commit_box_stops_take_text_focus_and_tab_still_leaves_them(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    keys(cx, "shift-tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
    });
    cx.simulate_input("body");
    cx.simulate_keystrokes("h l");
    keys(cx, "shift-tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitSummary));
    });
    cx.simulate_input("summary");
    settle_visual(cx);
    view.read_with(cx, |view, cx| {
        assert_eq!(view.summary_input().read(cx).text(), "summary");
        assert_eq!(view.description_input().read(cx).text(), "bodyhl");
    });

    keys(cx, "tab tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::Sidebar);
    });
}

#[gpui::test]
fn find_keeps_its_keys_after_a_control_had_focus(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    keys(cx, "tab tab");
    let was_tree_mode = tree_mode(cx);
    view.update(cx, |view, cx| view.open_find(cx));
    keys(cx, "tab space");
    assert_eq!(tree_mode(cx), was_tree_mode);
    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some(" "));
        assert_eq!(view.focused_control(), None);
    });
    keys(cx, "escape");
    view.read_with(cx, |view, _| assert!(view.find_query_text().is_none()));
}

#[gpui::test]
fn row_clicks_and_diff_edit_release_the_focused_control(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    keys(cx, "tab tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::TreeToggle));
    });

    let row = cx.debug_bounds("file-row-0").expect("first file row");
    cx.simulate_click(row.center(), Modifiers::default());
    settle_visual(cx);
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert_eq!(view.active_pane(), ActivePane::FileColumn);
    });

    keys(cx, "tab");
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.read_with(cx, |view, _| assert_eq!(view.focused_control(), None));

    keys(cx, "tab");
    view.read_with(cx, |view, _| assert_eq!(view.focused_control(), None));
}

#[gpui::test]
fn text_focus_overrides_the_previous_tab_stop(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    keys(cx, "tab tab");
    let was_tree_mode = tree_mode(cx);
    let summary = view.read_with(cx, |view, _| view.summary_input().clone());
    cx.focus(&summary);
    keys(cx, "space");
    assert_eq!(tree_mode(cx), was_tree_mode);
    keys(cx, "tab");
    view.read_with(cx, |view, cx| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
        assert_eq!(view.summary_input().read(cx).text(), " ");
    });

    cx.focus(&summary);
    keys(cx, "tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::CommitDescription));
    });
}

#[gpui::test]
fn keyboard_opened_filters_submit_and_release_text_focus(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    tab_to(&view, cx, FocusStop::FilterToggle);
    keys(cx, "space enter");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), None);
        assert!(view.file_filter_visible());
    });
    keys(cx, "escape");
    assert!(!view.read_with(cx, |view, _| view.file_filter_visible()));

    tab_to(&view, cx, FocusStop::RevsetFilter);
    keys(cx, "space");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::RevsetFilter));
        assert!(view.revset_editor_text().is_some());
    });
    keys(cx, "tab");
    view.read_with(cx, |view, _| {
        assert_eq!(view.focused_control(), Some(FocusStop::FileManager));
        assert!(view.revset_editor_text().is_none());
    });
}

#[gpui::test]
fn a_keyboard_opened_revset_popup_applies_on_return(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);

    tab_to(&view, cx, FocusStop::RevsetPresets);
    keys(cx, "enter");
    assert!(view.read_with(cx, |view, _| view.revset_popup_open()));
    cx.simulate_input("all()");
    keys(cx, "enter");
    view.read_with(cx, |view, cx| {
        assert!(!view.revset_popup_open());
        assert_eq!(view.view_model().read(cx).revset(), "all()");
    });
}

#[gpui::test]
fn edit_file_follows_edit_diff_in_the_tab_cycle(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    std::fs::write(fixture.path.join("notes.md"), "notes\n").expect("write working-copy file");
    let (view, cx) = open_focused(&fixture, cx);
    select_file(&view, "notes.md", cx);

    tab_to(&view, cx, FocusStop::EditDiff);
    keys(cx, "tab");
    assert_eq!(
        view.read_with(cx, |view, _| view.focused_control()),
        Some(FocusStop::EditFile)
    );
    keys(cx, "space");
    assert!(view.read_with(cx, |view, _| view.file_editor_active()));
}

fn tab_to(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, stop: FocusStop) {
    for _ in 0..24 {
        if view.read_with(cx, |view, _| view.focused_control()) == Some(stop) {
            return;
        }
        keys(cx, "tab");
    }
    panic!("{stop:?} is not in the Tab cycle");
}

#[gpui::test]
fn change_list_keys_follow_the_selected_lane(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    jj_test::run_jj_in(&fixture.path, &["new", "-m", "left"]);
    jj_test::run_jj_in(&fixture.path, &["new", "@-", "-m", "right"]);
    let (view, cx) = open_focused(&fixture, cx);
    let row = |cx: &VisualTestContext, subject: &str| {
        let commit_id = change_with_subject(&view, cx, subject).commit_id;
        view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            vm.graph
                .changes
                .iter()
                .position(|change| change.commit_id == commit_id)
        })
    };
    let selected =
        |cx: &VisualTestContext| view.read_with(cx, |view, cx| view.view_model().read(cx).selected);
    let right = row(cx, "right").expect("right row");
    let left = row(cx, "left").expect("left row");
    let parent = left + 1;
    assert!(right < left);

    view.update(cx, |view, cx| view.select_change(right, cx));
    cx.simulate_keystrokes("j");
    settle_visual(cx);
    assert_eq!(selected(cx), Some(parent));
    cx.simulate_keystrokes("up");
    settle_visual(cx);
    assert_eq!(selected(cx), Some(right));

    let pane = |cx: &VisualTestContext| view.read_with(cx, |view, _| view.active_pane());
    cx.simulate_keystrokes("l");
    settle_visual(cx);
    assert_eq!((selected(cx), pane(cx)), (Some(left), ActivePane::Sidebar));
    cx.simulate_keystrokes("right");
    settle_visual(cx);
    assert_eq!((selected(cx), pane(cx)), (Some(left), ActivePane::Sidebar));
    cx.simulate_keystrokes("h h");
    settle_visual(cx);
    assert_eq!((selected(cx), pane(cx)), (Some(right), ActivePane::Sidebar));

    view.update(cx, |view, cx| view.select_change(left, cx));
    cx.simulate_keystrokes("j");
    settle_visual(cx);
    assert_eq!(selected(cx), Some(left));
}

#[gpui::test]
fn closing_a_modal_returns_keys_to_the_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    cx.simulate_resize(size(px(1600.), px(1000.)));
    for dismiss in ["text-modal-cancel", "text-modal-primary", "pr-import"] {
        cx.simulate_keystrokes("j");
        settle_visual(cx);
        let parent = view.read_with(cx, |view, cx| view.view_model().read(cx).selected);
        if dismiss == "pr-import" {
            view.update(cx, |view, cx| view.open_pr_import(cx));
            settle_visual(cx);
            cx.simulate_keystrokes("escape");
        } else {
            let pencil = cx.debug_bounds("edit-description").unwrap();
            cx.simulate_click(pencil.center(), Modifiers::default());
            settle_visual(cx);
            let button = cx.debug_bounds(dismiss).unwrap();
            cx.simulate_click(button.center(), Modifiers::default());
        }
        settle_visual(cx);
        assert!(!view.read_with(cx, |view, _| view.has_text_modal()
            || view.has_pr_import_modal()));

        cx.simulate_keystrokes("k");
        settle_visual(cx);
        assert_ne!(
            view.read_with(cx, |view, cx| view.view_model().read(cx).selected),
            parent,
            "{dismiss}"
        );
    }
}
