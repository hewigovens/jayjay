use std::fs;

use gpui::{Entity, TestAppContext, VisualTestContext};
use jayjay_core::diff::DiffSpanStyle;
use jayjay_core::{DiffEditDestination, Repo};
use jayjay_gpui::repo::window::RepoWindow;
use jj_test::{LinearFixture, run_jj_in};

pub(super) use crate::support::settle_visual;
use crate::support::{install_test_globals, load_selected_change_files};

pub(super) fn enter_and_select_line(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    text: &str,
) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == text)
            .unwrap_or_else(|| panic!("added line '{text}' present"));
        (
            vm.selected_hunk().expect("hunk selected").path.clone(),
            line,
        )
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, line as u32 + 1, cx);
    });
}

pub(super) fn enter_and_select_group(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    added_text: &str,
) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .expect("diff loaded")
            .lines
            .iter()
            .position(|line| line.style == DiffSpanStyle::Added && line.text() == added_text)
            .unwrap_or_else(|| panic!("added line '{added_text}' present"));
        (
            vm.selected_hunk().expect("hunk selected").path.clone(),
            line,
        )
    });
    view.update_in(cx, |view, _, cx| view.enter_diff_edit(cx));
    settle_visual(cx);
    view.update_in(cx, |view, _, cx| {
        view.select_diff_edit_display_group(&path, line as u32 + 1, cx);
    });
}

pub(super) fn apply_with_message(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    destination: DiffEditDestination,
    message: &str,
) {
    view.update_in(cx, |view, _, cx| {
        view.set_diff_edit_message(message, cx);
        view.start_diff_edit_apply(destination, cx);
    });
    settle_visual(cx);
}

pub(super) fn selected_change_id(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .selected_change()
            .expect("change selected")
            .change_id
            .id
            .clone()
    })
}

pub(super) fn select_change_by_description(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    description: &str,
) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .graph
            .changes
            .iter()
            .position(|change| change.description.trim() == description)
            .expect("fixture change present")
    });
    view.update_in(cx, |view, _, cx| view.select_change(ix, cx));
    settle_visual(cx);
}

pub(super) fn select_file_by_path(
    view: &Entity<RepoWindow>,
    cx: &mut VisualTestContext,
    path: &str,
) {
    let ix = view.read_with(cx, |view, cx| {
        view.view_model()
            .read(cx)
            .files
            .as_ref()
            .expect("files loaded")
            .iter()
            .position(|hunk| hunk.path == path)
            .unwrap_or_else(|| panic!("file '{path}' present"))
    });
    view.update_in(cx, |view, _, cx| view.select_file(ix, cx));
    settle_visual(cx);
}

pub(super) fn append_unloaded_file(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    view.update_in(cx, |view, _, cx| {
        view.view_model().update(cx, |vm, _| {
            let files = std::sync::Arc::make_mut(vm.files.as_mut().expect("files loaded"));
            let mut pending = files.first().expect("changed file").clone();
            pending.path = "still-loading.txt".to_owned();
            files.push(pending);
        });
    });
}

pub(super) fn assert_toast(view: &Entity<RepoWindow>, cx: &mut VisualTestContext, expected: &str) {
    assert_eq!(
        view.read_with(cx, |view, _| view.toast().map(|toast| toast.to_string())),
        Some(expected.to_owned())
    );
}

pub(super) fn separated_edits_fixture(with_child: bool) -> LinearFixture {
    let fixture = LinearFixture::build();
    let base = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n";
    fs::write(fixture.path.join("edit.txt"), base).expect("write base file");
    run_jj_in(&fixture.path, &["describe", "-m", "edit base"]);
    run_jj_in(&fixture.path, &["new", "-m", "edit source"]);
    fs::write(
        fixture.path.join("edit.txt"),
        "one\nselected two\nthree\nfour\nfive\nsix\nseven\nremaining eight\nnine\nten\n",
    )
    .expect("write separated edits");
    run_jj_in(&fixture.path, &["st"]);
    if with_child {
        run_jj_in(&fixture.path, &["new", "-m", "working child"]);
        fs::write(fixture.path.join("working.txt"), "working edit\n")
            .expect("write working-copy edit");
        run_jj_in(&fixture.path, &["st"]);
    }
    fixture
}

pub(super) fn two_file_edits_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    fs::write(fixture.path.join("edit.txt"), "one\ntwo\nthree\nfour\n").unwrap();
    fs::write(fixture.path.join("untouched.txt"), "alpha\nbeta\ngamma\n").unwrap();
    run_jj_in(&fixture.path, &["describe", "-m", "base files"]);
    run_jj_in(&fixture.path, &["new"]);
    fs::write(
        fixture.path.join("edit.txt"),
        "one\nselected two\nthree\nunselected four\n",
    )
    .unwrap();
    fs::write(
        fixture.path.join("untouched.txt"),
        "alpha\nchanged beta\ngamma\n",
    )
    .unwrap();
    run_jj_in(&fixture.path, &["st"]);
    fixture
}

pub(super) fn two_file_working_copy_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nchanged\n",
    )
    .expect("write README edit");
    fs::write(fixture.path.join("feature.txt"), "feature\nchanged\n").expect("write feature edit");
    run_jj_in(&fixture.path, &["st"]);
    fixture
}

pub(super) fn change_by_description(repo: &Repo, description: &str) -> jayjay_core::ChangeInfo {
    repo.log("all()")
        .expect("load graph")
        .into_iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("change '{description}' present"))
}

pub(super) fn change_by_id(repo: &Repo, change_id: &str) -> jayjay_core::ChangeInfo {
    repo.log(change_id)
        .expect("load change")
        .into_iter()
        .next()
        .expect("change present")
}

pub(super) fn select_first_changed_line(view: &Entity<RepoWindow>, cx: &mut VisualTestContext) {
    let (path, line) = view.read_with(cx, |view, cx| {
        let vm = view.view_model().read(cx);
        let line = vm
            .current_diff
            .as_ref()
            .unwrap()
            .lines
            .iter()
            .position(|line| line.is_changed())
            .unwrap();
        (vm.selected_hunk().unwrap().path.clone(), line)
    });
    view.update_in(cx, |view, _, cx| {
        view.toggle_diff_edit_display_line(&path, line as u32 + 1, cx)
    });
}

pub(super) fn open_changed_repo(
    cx: &mut TestAppContext,
) -> (LinearFixture, Entity<RepoWindow>, &mut VisualTestContext) {
    let fixture = LinearFixture::build();
    fs::write(
        fixture.path.join("README.md"),
        "# Sample project\nfirst edit\nsecond edit\n",
    )
    .unwrap();
    let (view, cx) = open_fixture(&fixture, cx);
    (fixture, view, cx)
}

pub(super) fn open_fixture<'a>(
    fixture: &LinearFixture,
    cx: &'a mut TestAppContext,
) -> (Entity<RepoWindow>, &'a mut VisualTestContext) {
    open_repo(fixture.path.clone(), cx)
}

pub(super) fn open_repo(
    path: std::path::PathBuf,
    cx: &mut TestAppContext,
) -> (Entity<RepoWindow>, &mut VisualTestContext) {
    install_test_globals(cx);
    let (view, cx) = cx.add_window_view(|_, cx| RepoWindow::new(path, cx));
    let cx: &mut VisualTestContext = cx;
    load_selected_change_files(&view, cx);
    settle_visual(cx);
    (view, cx)
}
