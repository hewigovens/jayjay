use crate::harness::*;
use gpui::{AppContext, Entity, TestAppContext};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

fn open_repo_window_view(fixture: &LinearFixture, cx: &mut TestAppContext) -> Entity<RepoWindow> {
    install_test_globals(cx);
    let view = cx.new(|cx| RepoWindow::new(fixture.path.clone(), cx));
    settle(cx); // repo opens async
    view
}

fn submit_workspace_name(view: &Entity<RepoWindow>, name: &str, cx: &mut TestAppContext) {
    view.update(cx, |view, cx| view.open_create_workspace(cx));
    let input = view
        .read_with(cx, |view, _| view.text_modal_input())
        .expect("modal input present once the overlay opens");
    input.update(cx, |input, cx| input.set_text(name.to_owned(), cx));
    view.update(cx, |view, cx| view.submit_text_modal(cx));
}

#[gpui::test]
fn new_workspace_modal_shows_sibling_destination(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = open_repo_window_view(&fixture, cx);

    view.update(cx, |view, cx| view.open_create_workspace(cx));

    view.read_with(cx, |view, _| {
        assert!(view.has_text_modal(), "modal should open");
        let subtitle = view.text_modal_subtitle().expect("destination hint");
        let parent = fixture
            .path
            .parent()
            .expect("fixture parent dir")
            .display()
            .to_string();
        assert!(
            subtitle.contains(&parent),
            "subtitle {subtitle:?} should show the sibling parent {parent:?}"
        );
    });
}

#[gpui::test]
fn create_workspace_adds_sibling_workspace_and_stamps_mutation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = open_repo_window_view(&fixture, cx);

    submit_workspace_name(&view, "feature-ws", cx);

    view.read_with(cx, |view, cx| {
        assert!(!view.has_text_modal(), "modal closes on submit");
        assert!(
            view.view_model()
                .read(cx)
                .last_internal_mutation_at
                .is_some(),
            "workspace add must stamp the fs-watcher echo-suppression window"
        );
    });
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(vm.error.is_none(), "workspace add errored: {:?}", vm.error);
        assert!(
            vm.graph.workspaces.iter().any(|ws| ws.name == "feature-ws"),
            "new workspace should appear in the refreshed list: {:?}",
            vm.graph.workspaces
        );
        assert_eq!(
            view.toast().as_deref(),
            Some("Created workspace feature-ws")
        );
    });
    let dest = fixture
        .path
        .parent()
        .expect("fixture parent dir")
        .join("feature-ws");
    assert!(
        dest.join(".jj").exists(),
        "sibling workspace directory should exist at {dest:?}"
    );
    // SwiftUI parity: the created workspace opens in its own window.
    assert_eq!(cx.update(|cx| cx.windows().len()), 1);
}

#[gpui::test]
fn duplicate_workspace_name_error_surfaces(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let existing = fixture
        .path
        .parent()
        .expect("fixture parent dir")
        .join("dupe");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "dupe",
            existing.to_str().expect("utf8 dest"),
        ],
    );
    let view = open_repo_window_view(&fixture, cx);

    submit_workspace_name(&view, "dupe", cx);
    settle(cx);

    view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        assert!(
            vm.error.is_some(),
            "duplicate workspace name must surface an error"
        );
    });
}

#[gpui::test]
fn invalid_workspace_name_keeps_modal_and_skips_mutation(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let view = open_repo_window_view(&fixture, cx);

    submit_workspace_name(&view, "../evil", cx);
    settle(cx);

    view.read_with(cx, |view, cx| {
        assert!(view.has_text_modal(), "invalid name keeps the modal open");
        assert!(
            view.toast()
                .is_some_and(|toast| toast.contains("Invalid workspace name")),
            "toast should explain the rejection"
        );
        let vm = view.view_model().read(cx);
        assert!(
            vm.last_internal_mutation_at.is_none(),
            "no mutation may run for a rejected name"
        );
    });
}

#[gpui::test]
fn create_workspace_without_open_repo_shows_toast(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    install_test_globals(cx);
    // Onboarding path: the window exists but no repo has been opened yet.
    let view = cx.new(|cx| RepoWindow::new_with_onboarding(fixture.path.clone(), cx));
    settle(cx);

    view.update(cx, |view, cx| view.open_create_workspace(cx));

    view.read_with(cx, |view, _| {
        assert!(!view.has_text_modal(), "no modal without an open repo");
        assert_eq!(view.toast().as_deref(), Some("Repository is not open"));
    });
}
