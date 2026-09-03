use jayjay_core::{MutationEffect, Repo};
use jj_test::{LinearFixture, run_jj_in};

#[test]
fn absorb_distinguishes_no_op_from_applied_changes() {
    let no_op = LinearFixture::build();
    let repo = Repo::open(&no_op.path).expect("open no-op fixture");

    assert_eq!(
        repo.absorb("@").expect("absorb new files"),
        MutationEffect::Unchanged
    );

    let applied = LinearFixture::build();
    run_jj_in(&applied.path, &["bookmark", "delete", "main"]);
    applied.add_tracked_working_copy_edits();
    let repo = Repo::open(&applied.path).expect("open absorbable fixture");

    assert_eq!(
        repo.absorb("@").expect("absorb tracked edits"),
        MutationEffect::Changed
    );
}
