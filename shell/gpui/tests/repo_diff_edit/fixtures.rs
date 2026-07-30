use std::fs;

use jayjay_core::Repo;
use jj_test::{LinearFixture, run_jj_in};

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
