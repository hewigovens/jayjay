use std::fs;
use std::sync::{Arc, Mutex};

use crate::harness::*;
use gpui::{Focusable, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use jayjay_gpui::app::actions::OpenCommandPalette;
use jayjay_gpui::app::config::{self, AppearanceMode};
use jayjay_gpui::app::links;
use jayjay_gpui::repo::RepoWindow;
use jayjay_gpui::windows::command_palette::CommandPalette;
use jj_test::{LinearFixture, run_jj_in};

#[gpui::test]
fn command_palette_ctrl_n_enter_dispatches_selected_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("theme");
    palette_cx.simulate_keystrokes("ctrl-n ctrl-n enter");

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

#[gpui::test]
fn command_palette_supports_line_editing_keys(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let (palette, cx) = cx.add_window_view(|_, cx| CommandPalette::new("".into(), None, cx));
    let cx: &mut VisualTestContext = cx;

    cx.focus(&palette);
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("|");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha beta |gamma");
    });

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-backspace");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha beta ");
    });

    cx.simulate_input("gamma");
    cx.simulate_keystrokes("ctrl-u");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "");
    });

    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("ctrl-a alt-right ctrl-k");

    palette.read_with(cx, |palette, _| {
        assert_eq!(palette.query_text(), "alpha");
    });
}

#[gpui::test]
fn command_palette_renders_input_caret(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    assert!(palette_cx.debug_bounds("command-palette-caret").is_some());
    palette_cx.simulate_input("jj status");
    assert!(palette_cx.debug_bounds("command-palette-caret").is_some());
}

#[gpui::test]
fn command_palette_mouse_click_dispatches_action(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx.simulate_input("dark");

    let row = palette_cx
        .debug_bounds("command-palette-action-theme-dark")
        .expect("theme dark row bounds");
    palette_cx.simulate_click(row.center(), Modifiers::default());

    assert_eq!(
        palette_cx.cx.read(|cx| config::current(cx).appearance),
        AppearanceMode::Dark
    );
}

#[gpui::test]
fn command_palette_help_topic_opens_guide_at_topic_anchor(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("help leave review notes");
    let row = palette_cx
        .debug_bounds("command-palette-help-review-notes")
        .expect("help topic row");
    palette_cx.simulate_click(row.center(), Modifiers::default());

    assert_eq!(
        crate::harness::opened_url(&mut palette_cx.cx).as_deref(),
        Some("https://jayjay.hewig.dev/guide.html#review-notes")
    );
}

#[gpui::test]
fn command_palette_user_guide_action_opens_canonical_guide_url(cx: &mut TestAppContext) {
    install_test_globals(cx);
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("user guide");
    palette_cx.simulate_keystrokes("enter");

    assert_eq!(
        crate::harness::opened_url(&mut palette_cx.cx).as_deref(),
        Some(jayjay_gpui::app::links::GUIDE_URL)
    );
}

#[gpui::test]
fn command_palette_send_feedback_dispatches_compose_url(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let opened_url = Arc::new(Mutex::new(None));
    cx.update({
        let opened_url = opened_url.clone();
        move |cx| {
            links::install_url_opener(cx, move |url| {
                *opened_url.lock().expect("opened URL lock") = Some(url.to_owned());
                true
            });
            CommandPalette::open("".into(), None, cx);
        }
    });
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);

    palette_cx.simulate_input("send feedback");
    palette_cx.simulate_keystrokes("enter");
    settle(&mut palette_cx.cx);

    assert_eq!(
        opened_url.lock().expect("opened URL lock").as_deref(),
        Some(links::FEEDBACK_URL)
    );
}

#[gpui::test]
fn find_bar_supports_line_editing_keys(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(fixture.path.clone(), cx));
    let cx: &mut VisualTestContext = cx;
    settle_visual(cx);

    view.update_in(cx, |view, window, cx| {
        view.open_find(cx);
        view.focus_handle(cx).focus(window, cx);
    });
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-left");
    cx.simulate_input("|");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some("alpha beta |gamma"));
    });

    cx.simulate_keystrokes("cmd-a");
    cx.simulate_input("alpha beta gamma");
    cx.simulate_keystrokes("alt-backspace");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some("alpha beta "));
    });

    cx.simulate_input("gamma");
    cx.simulate_keystrokes("ctrl-u");

    view.read_with(cx, |view, _| {
        assert_eq!(view.find_query_text(), Some(""));
    });
}

#[gpui::test]
fn open_command_palette_action_opens_the_palette_from_the_repo_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    repo_cx.focus(&view);
    repo_cx.dispatch_action(OpenCommandPalette);
    settle_visual(repo_cx);
    assert!(
        repo_cx
            .cx
            .windows()
            .iter()
            .any(|window| window.downcast::<CommandPalette>().is_some())
    );
}

fn open_palette_for(view: &gpui::Entity<RepoWindow>, cx: &mut TestAppContext) -> VisualTestContext {
    let view = view.clone();
    cx.update(|cx| CommandPalette::open("".into(), Some(view), cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx
}

#[gpui::test]
fn palette_lists_switchable_workspaces_and_filters_by_name(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    add_workspace(&fixture, "second");
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());

    palette_cx.simulate_input("switch");
    settle_visual(&mut palette_cx);
    assert!(
        palette_cx
            .debug_bounds("command-palette-workspace-first")
            .is_some()
    );
    assert!(
        palette_cx
            .debug_bounds("command-palette-workspace-second")
            .is_some()
    );
    assert!(
        palette_cx
            .debug_bounds("command-palette-workspace-default")
            .is_none(),
        "the current workspace is not a switch target"
    );

    palette_cx.simulate_input(" sec");
    settle_visual(&mut palette_cx);
    assert!(
        palette_cx
            .debug_bounds("command-palette-workspace-second")
            .is_some()
    );
    assert!(
        palette_cx
            .debug_bounds("command-palette-workspace-first")
            .is_none()
    );
}

#[gpui::test]
fn palette_workspace_entry_switches_the_current_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    let (view, repo_cx) = open_fixture(&fixture, cx);
    {
        let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());
        palette_cx.simulate_input("switch first");
        palette_cx.simulate_keystrokes("enter");
        settle_visual(&mut palette_cx);
    }
    settle_visual(repo_cx);

    assert_eq!(current_workspace(&view, repo_cx), "first");
    assert_eq!(repo_window_count(repo_cx), 1);
}

#[gpui::test]
fn command_palette_zoom_entries_change_the_font_size(cx: &mut TestAppContext) {
    install_test_globals(cx);
    let baseline = font_sizes(cx);

    {
        let mut palette_cx = open_bare_palette(cx);
        click_palette_row(&mut palette_cx, "zoom in", "command-palette-action-zoom-in");
    }
    assert_eq!(font_sizes(cx), (baseline.0 + 1., baseline.1 + 1.));

    {
        let mut palette_cx = open_bare_palette(cx);
        click_palette_row(
            &mut palette_cx,
            "zoom out",
            "command-palette-action-zoom-out",
        );
    }
    assert_eq!(font_sizes(cx), baseline);

    {
        let mut palette_cx = open_bare_palette(cx);
        click_palette_row(&mut palette_cx, "zoom in", "command-palette-action-zoom-in");
    }
    {
        let mut palette_cx = open_bare_palette(cx);
        click_palette_row(
            &mut palette_cx,
            "reset zoom",
            "command-palette-action-reset-zoom",
        );
    }
    assert_eq!(font_sizes(cx), baseline);
}

#[gpui::test]
fn command_palette_new_child_creates_a_change_on_the_selection(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    let (before, parent_commit) = view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let selected = vm.selected_change().expect("selected change");
        (vm.graph.changes.len(), selected.commit_id.id.clone())
    });

    {
        let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());
        click_palette_row(&mut palette_cx, "new child", "command-palette-new-child");
    }
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "new child errored: {:?}", vm.error);
        assert_eq!(vm.graph.changes.len(), before + 1);
        let selected = vm.selected_change().expect("selected change");
        assert!(selected.is_working_copy);
        assert_eq!(selected.parents.first(), Some(&parent_commit));
    });
}

#[gpui::test]
fn command_palette_revset_preset_applies_the_filter(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    {
        let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());
        click_palette_row(
            &mut palette_cx,
            "show conflicts",
            "command-palette-revset-conflicts",
        );
    }
    settle_visual(repo_cx);

    view.read_with(repo_cx, |view, cx| {
        assert_eq!(view.view_model().read(cx).revset(), "conflicts()");
    });
}

#[gpui::test]
fn command_palette_expands_every_unmodified_region(cx: &mut TestAppContext) {
    let fixture = two_region_fixture();
    let (view, repo_cx) = open_fixture(&fixture, cx);
    select_file(&view, "context.txt", repo_cx);
    let before = collapsed_region_count(&view, repo_cx);
    assert!(
        before >= 2,
        "fixture has more than one collapsed region, got {before}"
    );

    {
        let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());
        click_palette_row(
            &mut palette_cx,
            "expand all unmodified",
            "command-palette-action-expand-all-unmodified-lines",
        );
    }
    settle_visual(repo_cx);

    assert_eq!(collapsed_region_count(&view, repo_cx), 0);
}

#[gpui::test]
fn command_palette_forget_and_delete_asks_for_confirmation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    add_workspace(&fixture, "first");
    let (view, repo_cx) = open_fixture(&fixture, cx);
    {
        let mut palette_cx = open_palette_for(&view, &mut repo_cx.cx.clone());
        click_palette_row(
            &mut palette_cx,
            "delete first",
            "command-palette-delete-first",
        );
    }
    settle_visual(repo_cx);

    assert!(repo_cx.debug_bounds("confirmation").is_some());
    assert!(
        repo_cx
            .debug_bounds("confirmation-dont-ask-again")
            .is_some(),
        "workspace delete reuses the confirmation that can be skipped"
    );
}

fn open_bare_palette(cx: &mut TestAppContext) -> VisualTestContext {
    cx.update(|cx| CommandPalette::open("".into(), None, cx));
    let window = cx.windows().last().copied().expect("palette window");
    let mut palette_cx = VisualTestContext::from_window(window, cx);
    settle_visual(&mut palette_cx);
    palette_cx
}

fn click_palette_row(cx: &mut VisualTestContext, query: &str, selector: &'static str) {
    cx.simulate_input(query);
    settle_visual(cx);
    let row = cx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing palette row {selector}"));
    cx.simulate_click(row.center(), Modifiers::default());
}

fn font_sizes(cx: &TestAppContext) -> (f32, f32) {
    cx.read(|cx| {
        let config = config::current(cx);
        (config.interface_font_size(), config.code_font_size())
    })
}

fn collapsed_region_count(view: &gpui::Entity<RepoWindow>, cx: &mut VisualTestContext) -> usize {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .shown
            .diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .filter(|line| line.context_region.is_some())
            .count()
    })
}

fn two_region_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    let base = (1..=90)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(fixture.path.join("context.txt"), &base).expect("write context base");
    run_jj_in(&fixture.path, &["new"]);
    let edited = base
        .replace("line 10", "changed early")
        .replace("line 70", "changed late");
    fs::write(fixture.path.join("context.txt"), edited).expect("write context edit");
    run_jj_in(&fixture.path, &["st"]);
    fixture
}
