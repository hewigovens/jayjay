use std::fs;

use jj_test::{LinearFixture, run_jj_in};

pub(super) fn context_fixture() -> LinearFixture {
    let fixture = LinearFixture::build();
    let base = (1..=80)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(fixture.path.join("context.txt"), &base).expect("write context base");
    run_jj_in(&fixture.path, &["new"]);

    let edited = base
        .replace("line 03", "changed near start")
        .replace("line 78", "changed near end");
    fs::write(fixture.path.join("context.txt"), edited).expect("write context edit");
    fs::write(
        fixture.path.join("other.txt"),
        "another working-copy file\n",
    )
    .expect("write second edit");
    run_jj_in(&fixture.path, &["st"]);
    fixture
}
