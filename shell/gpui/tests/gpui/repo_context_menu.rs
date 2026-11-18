use crate::harness::*;
use gpui::{
    Entity, Focusable, KeyDownEvent, Keystroke, Modifiers, MouseButton, TestAppContext,
    VisualTestContext, px,
};
use jayjay_gpui::app::config;
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

const NEW_CHANGE: &str = "New Change on Top";
const COPY_PATH: &str = "Copy Path";

#[gpui::test]
fn keyboard_change_menu_targets_the_navigated_change(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    keys(cx, "j");
    let parent = selected_commit_id(&view, cx);
    keys(cx, "shift-f10");
    assert!(menu_shows(cx, NEW_CHANGE));
    keys(cx, "j escape");
    assert!(!menu_shows(cx, NEW_CHANGE));
    assert_eq!(selected_commit_id(&view, cx), parent);
    keys(cx, "menu end");
    for _ in 0..3 {
        if menu_shows(cx, "Duplicate") {
            break;
        }
        keys(cx, "up right");
    }
    assert!(menu_shows(cx, "Duplicate"));
    keys(cx, "escape");
    assert!(!menu_shows(cx, "Duplicate"));
    assert!(menu_shows(cx, NEW_CHANGE));
    keys(cx, "escape menu d");
    assert!(
        view.read_with(cx, |view, _| view.has_text_modal()),
        "D opens the description editor"
    );
    let cancel = cx.debug_bounds("text-modal-cancel").unwrap();
    cx.simulate_click(cancel.center(), Modifiers::default());
    keys(cx, "ctrl-enter n");
    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "{:?}", vm.error);
        assert_eq!(vm.selected_change().unwrap().parents, vec![parent]);
    });
    assert!(!menu_shows(cx, NEW_CHANGE));
}

#[gpui::test]
fn keyboard_file_menu_copies_the_selected_path_in_both_layouts(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_focused(&fixture, cx);
    keys(cx, "tab j");
    let path = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        vm.files.as_ref().unwrap()[vm.selected_file_ix.unwrap()]
            .path
            .clone()
    });
    for (tree, opener) in [(false, "shift-f10"), (true, "menu")] {
        set_tree_file_list(cx, tree);
        keys(cx, opener);
        assert!(menu_shows(cx, COPY_PATH));
        keys(
            cx,
            if tree {
                "home down down down down enter"
            } else {
                "c"
            },
        );
        assert_eq!(clipboard_text(cx), Some(path.clone()));
        assert!(!menu_shows(cx, COPY_PATH));
    }
    focus_summary(&view, cx);
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            key: "menu".into(),
            ..Default::default()
        },
        is_held: false,
        prefer_character_input: false,
    });
    keys(cx, "shift-f10 c");
    assert!(!menu_shows(cx, COPY_PATH));
    assert_eq!(summary_text(&view, cx), "c");
    set_tree_file_list(cx, false);
    let row = cx.debug_bounds("file-row-1").unwrap();
    cx.simulate_mouse_down(row.center(), MouseButton::Right, Modifiers::default());
    settle_visual(cx);
    assert!(menu_shows(cx, COPY_PATH));
    focus_summary(&view, cx);
    settle_visual(cx);
    keys(cx, "down up c");
    assert_eq!(clipboard_text(cx), Some(path));
    assert_eq!(summary_text(&view, cx), "c");
    assert!(!menu_shows(cx, COPY_PATH));
    view.update_in(cx, |view, window, cx| {
        assert!(
            view.summary_input()
                .read(cx)
                .focus_handle(cx)
                .is_focused(window),
            "summary should retain focus"
        );
    });
    keys(cx, "c");
    assert_eq!(summary_text(&view, cx), "cc");
}

#[gpui::test]
fn an_open_menu_does_not_intercept_another_windows_typing(cx: &mut TestAppContext) {
    let first_fixture = LinearFixture::build();
    let (first, selected) = {
        let (view, cx) = open_focused(&first_fixture, cx);
        keys(cx, "menu");
        assert!(menu_shows(cx, NEW_CHANGE));
        let selected = selected_commit_id(&view, cx);
        (view, selected)
    };
    let second_fixture = LinearFixture::build();
    let (second, cx) = open_fixture(&second_fixture, cx);
    focus_summary(&second, cx);
    keys(cx, "n");
    assert_eq!(summary_text(&second, cx), "n");
    assert_eq!(selected_commit_id(&first, cx), selected);
}

#[gpui::test]
fn keyboard_menu_reveals_a_selected_change_scrolled_out_of_view(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    for n in 0..30 {
        run_jj_in(&fixture.path, &["new", "-m", &format!("filler {n}")]);
    }
    let (view, cx) = open_focused(&fixture, cx);
    let row_selector = selector(format!("dag-change-{}", selected_commit_id(&view, cx)));
    let row = cx
        .debug_bounds(row_selector)
        .expect("selected change starts visible");
    scroll_wheel(cx, row.center(), px(-5000.));
    assert!(
        cx.debug_bounds(row_selector).is_none(),
        "selected change scrolled out of view"
    );

    keys(cx, "shift-f10");
    let row = cx
        .debug_bounds(row_selector)
        .expect("opening the menu reveals the selected change");
    let menu = cx
        .debug_bounds("context-menu-New Change on Top")
        .expect("menu opened");
    assert!(menu.top() >= row.top(), "menu anchors to the revealed row");
}

#[gpui::test]
fn forge_actions_are_offered_only_for_supported_forges(cx: &mut TestAppContext) {
    for (remote, supported) in [
        ("https://forge.example/team/repo.git", false),
        ("https://github.com/example/repo.git", true),
    ] {
        let fixture = LinearFixture::build();
        fixture.add_origin(remote);
        run_jj_in(
            &fixture.path,
            &["bookmark", "create", "feature", "-r", "@-"],
        );
        let (view, cx) = open_fixture(&fixture, cx);

        let stacked_prs = view.read_with(cx, |view, cx| {
            let vm = view.view_model().read(cx);
            let change = vm.graph.changes.iter().find(|change| !change.is_immutable);
            view.build_change_menu(change.expect("mutable change"), cx)
                .iter()
                .any(|item| item.label.as_ref() == "Create / Update Stacked PRs…")
        });
        assert_eq!(stacked_prs, supported, "{remote}: stacked PRs");

        let chip = cx
            .debug_bounds("dag-bookmark-feature")
            .expect("feature bookmark chip");
        cx.simulate_mouse_down(chip.center(), MouseButton::Right, Modifiers::default());
        settle_visual(cx);
        assert!(menu_shows(cx, "Push"), "{remote}: Push must stay available");
        assert_eq!(
            menu_shows(cx, "Pull Request on GitHub"),
            supported,
            "{remote}: pull request"
        );
    }
}

fn focus_summary(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, window, cx| {
        view.summary_input()
            .read(cx)
            .focus_handle(cx)
            .focus(window, cx)
    });
}

fn summary_text(view: &Entity<RepoWindow>, cx: &VisualTestContext) -> String {
    view.read_with(cx, |view, cx| {
        view.summary_input().read(cx).text().to_string()
    })
}

fn set_tree_file_list(cx: &mut VisualTestContext, tree: bool) {
    cx.update(|_, cx| config::update(cx, |config| config.diff.tree_file_list = tree));
    settle_visual(cx);
}

#[gpui::test]
fn the_change_menu_widens_with_the_interface_zoom(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (_view, cx) = open_focused(&fixture, cx);
    let menu_width = |cx: &mut VisualTestContext| {
        keys(cx, "shift-f10");
        let width = pane_width(cx, selector(format!("context-menu-{NEW_CHANGE}")));
        keys(cx, "escape");
        width
    };
    let font_size =
        |cx: &mut VisualTestContext| cx.cx.update(|cx| config::current(cx).interface_font_size());
    let (width, size) = (menu_width(cx), font_size(cx));
    zoom_to_max(cx);
    let (zoomed, zoomed_size) = (menu_width(cx), font_size(cx));
    assert!(
        zoomed >= width * zoomed_size / size * 0.8,
        "the labels grow with the zoom but {width} at {size}pt grew only to {zoomed} at {zoomed_size}pt"
    );
}
