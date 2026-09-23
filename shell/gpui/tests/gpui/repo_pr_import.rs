use std::cell::Cell;
use std::rc::Rc;

use crate::harness::{open_repo, settle_visual};
use gpui::{Entity, Modifiers, TestAppContext, VisualContext, VisualTestContext};
use jayjay_core::{
    PrState, PullRequestImportPreview, PullRequestImportRemote, PullRequestImportSource,
    PullRequestImportWorkspace,
};
use jayjay_gpui::repo::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

fn open_pr_import<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    let (view, cx) = open_repo(fixture.path.clone(), cx);
    cx.focus(&view);
    view.update_in(cx, |view, _, cx| view.open_pr_import(cx));
    settle_visual(cx);
    (view, cx)
}

fn show_preview(
    view: &Entity<RepoWindow>,
    existing_workspace: Option<PullRequestImportWorkspace>,
    cx: &mut VisualTestContext,
) {
    view.update_in(cx, |view, _, cx| {
        view.pr_import_show_preview_for_test(test_preview(existing_workspace), cx);
    });
    settle_visual(cx);
}

fn test_preview(
    existing_workspace: Option<PullRequestImportWorkspace>,
) -> PullRequestImportPreview {
    PullRequestImportPreview {
        pull_request: PullRequestImportSource {
            host: "GitHub".into(),
            base_repo: "hewigovens/jayjay".into(),
            number: 300,
            state: PrState::Open,
            title: "feat(workspace): new workspace from a pull request".into(),
            url: "https://github.com/hewigovens/jayjay/pull/300".into(),
        },
        remote: PullRequestImportRemote {
            name: "alice".into(),
            url: "https://github.com/alice/jayjay.git".into(),
            exists: false,
            bookmark: "pr-300".into(),
        },
        head_commit_id: "0123456789abcdef0123456789abcdef01234567".into(),
        same_repository: false,
        workspace: PullRequestImportWorkspace {
            name: "pr-300".into(),
            dest: "/tmp/pr-300".into(),
        },
        existing_workspace,
    }
}

#[gpui::test]
fn pr_import_unsupported_url_keeps_url_and_shows_inline_error(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    let input = view
        .read_with(cx, |view, _| view.pr_import_url_input())
        .expect("url input");
    input.update(cx, |input, cx| input.set_text("not a url".to_owned(), cx));
    let notified = Rc::new(Cell::new(false));
    let flag = notified.clone();
    cx.update(|_, cx| cx.observe(&view, move |_, _| flag.set(true)).detach());
    view.update(cx, |view, cx| view.submit_pr_import(cx));
    assert!(
        input.read_with(cx, |input, _| input.is_read_only()),
        "the URL is locked while it resolves"
    );
    assert!(notified.get(), "Resolve must re-render as Resolving…");
    settle_visual(cx);

    view.read_with(cx, |view, cx| {
        assert!(view.has_pr_import_modal(), "an error keeps the modal open");
        assert!(!input.read(cx).is_read_only());
        let error = view.pr_import_error().expect("inline error");
        assert!(
            error.contains("Not a supported pull request URL"),
            "unexpected error: {error}"
        );
        assert!(
            view.view_model().read(cx).error.is_none(),
            "the modal owns the error; nothing goes to the global error overlay"
        );
        assert_eq!(input.read(cx).text(), "not a url");
    });
}

#[gpui::test]
fn pr_import_typing_a_url_rerenders_the_modal(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    let notified = Rc::new(Cell::new(false));
    let flag = notified.clone();
    cx.update(|_, cx| cx.observe(&view, move |_, _| flag.set(true)).detach());

    cx.simulate_input("https://github.com/hewigovens/jayjay/pull/300");
    settle_visual(cx);

    assert!(
        notified.get(),
        "the Resolve button's enabled state lives in the parent render"
    );
    assert!(cx.debug_bounds("pr-import-resolve").is_some());
}

#[gpui::test]
fn pr_import_escape_cancels_url_stage(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    assert!(cx.debug_bounds("pr-import-url").is_some());

    cx.simulate_keystrokes("escape");
    settle_visual(cx);

    view.read_with(cx, |view, _| assert!(!view.has_pr_import_modal()));
}

#[gpui::test]
fn pr_import_preview_stage_seeds_workspace_fields(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    show_preview(&view, None, cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.pr_import_workspace_fields(cx),
            Some(("pr-300".to_owned(), "/tmp/pr-300".to_owned()))
        );
    });
    for selector in ["pr-import-name", "pr-import-dest", "pr-import-create"] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector} renders");
    }
    assert!(
        cx.debug_bounds("pr-import-url").is_none(),
        "the preview replaces the url stage"
    );
}

#[gpui::test]
fn pr_import_refreshed_preview_keeps_edited_fields(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    show_preview(&view, None, cx);
    cx.simulate_input("-mine");
    cx.simulate_keystrokes("tab");
    cx.simulate_input("-here");
    show_preview(&view, None, cx);

    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.pr_import_workspace_fields(cx),
            Some(("pr-300-mine".to_owned(), "/tmp/pr-300-here".to_owned()))
        );
    });
}

#[gpui::test]
fn pr_import_existing_workspace_offers_import_again(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let (view, cx) = open_pr_import(&fixture, cx);
    let existing = PullRequestImportWorkspace {
        name: "pr-300".into(),
        dest: fixture.path.display().to_string(),
    };
    show_preview(&view, Some(existing), cx);
    assert!(cx.debug_bounds("pr-import-open").is_some());
    assert!(cx.debug_bounds("pr-import-name").is_none());

    let again = cx
        .debug_bounds("pr-import-again")
        .expect("import again button");
    cx.simulate_click(again.center(), Modifiers::default());
    settle_visual(cx);

    assert!(cx.debug_bounds("pr-import-create").is_some());
    assert!(cx.debug_bounds("pr-import-open").is_none());
    assert!(cx.debug_bounds("pr-import-name").is_some());
}

#[gpui::test]
fn pr_import_open_workspace_closes_modal_and_opens_window(cx: &mut TestAppContext) {
    let fixture = LinearFixture::build();
    let existing_dest = fixture
        .path
        .parent()
        .expect("fixture parent dir")
        .join("pr-300");
    run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "pr-300",
            existing_dest.to_str().expect("utf8 dest"),
        ],
    );
    let (view, cx) = open_pr_import(&fixture, cx);
    let existing = PullRequestImportWorkspace {
        name: "pr-300".into(),
        dest: existing_dest.to_string_lossy().into_owned(),
    };
    show_preview(&view, Some(existing), cx);

    // An entity-only update, as when the async import finishes.
    view.update(cx, |view, cx| view.submit_pr_import(cx));
    settle_visual(cx);

    view.read_with(cx, |view, _| assert!(!view.has_pr_import_modal()));
    assert_eq!(cx.cx.update(|cx| cx.windows().len()), 2);
}
