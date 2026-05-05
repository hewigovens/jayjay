//! Component tests for `RepoViewModel`.
//!
//! Pattern: spin up a `TestAppContext`, build a `RepoViewModel` against a real
//! jj fixture, and assert state. This is the layer that catches write-milestone
//! regressions — refresh propagation, mutation flows, async loaders — without
//! needing pixels or a platform layer.
//!
//! Add new tests here (or in sibling files) as new view-model behavior lands.

mod support;

use gpui::{AppContext, TestAppContext};
use jayjay_gpui::repo::view_model::RepoViewModel;
use support::{SimpleFixture, jj_available};

#[gpui::test]
fn opens_simple_fixture_with_working_copy_selected(cx: &mut TestAppContext) {
    if !jj_available() {
        eprintln!("skipping: `jj`/`git` not on PATH");
        return;
    }
    let fixture = SimpleFixture::build();

    let vm = cx.new(|_| RepoViewModel::new(fixture.path.clone()));

    vm.read_with(cx, |vm, _| {
        assert!(vm.error.is_none(), "open errored: {:?}", vm.error);
        assert!(vm.repo.is_some(), "repo handle should be populated");
        assert!(
            vm.graph.entries.len() >= 4,
            "simple fixture should expose at least 4 changes (initial, hello, feature, wc), got {}",
            vm.graph.entries.len()
        );
        let selected_ix = vm.selected.expect("working copy should be selected");
        let selected = &vm.graph.entries[selected_ix].change;
        assert!(
            selected.is_working_copy,
            "selected change should be the working copy, got {:?}",
            selected.change_id
        );
    });
}
