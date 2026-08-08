use std::fs;
use std::path::{Path, PathBuf};

use plist::Value;

use crate::linear::LinearFixture;
use crate::run_jj_in;

/// The standard linear fixture plus structured files for Rust projection checks; the UI fixture script creates the separate app-facing repo.
pub struct FormatFixture {
    _linear: LinearFixture,
    pub path: PathBuf,
}

impl FormatFixture {
    pub const NOTEBOOK: &'static str = "analysis.ipynb";
    pub const MARKDOWN: &'static str = "notes.md";
    const HTML: &'static str = "release.html";
    pub const PLIST: &'static str = "Info.plist";
    pub const XML_PLIST: &'static str = "PlainInfo.plist";
    pub const CSV: &'static str = "data.csv";
    pub const SARIF: &'static str = "results.sarif";

    pub fn build() -> Self {
        let linear = LinearFixture::build();
        let path = linear.path.clone();
        write_format_files(&path);
        run_jj_in(&path, &["st"]);
        Self {
            _linear: linear,
            path,
        }
    }
}

fn write_format_files(repo: &Path) {
    copy_fixture(FormatFixture::NOTEBOOK, repo.join(FormatFixture::NOTEBOOK));
    copy_fixture(FormatFixture::MARKDOWN, repo.join(FormatFixture::MARKDOWN));
    copy_fixture(FormatFixture::HTML, repo.join(FormatFixture::HTML));
    copy_fixture(FormatFixture::CSV, repo.join(FormatFixture::CSV));
    copy_fixture(FormatFixture::SARIF, repo.join(FormatFixture::SARIF));
    copy_fixture(
        FormatFixture::XML_PLIST,
        repo.join(FormatFixture::XML_PLIST),
    );

    Value::from_file(fixture_path(FormatFixture::PLIST))
        .expect("read XML plist fixture")
        .to_file_binary(repo.join(FormatFixture::PLIST))
        .expect("write binary plist");
}

fn copy_fixture(name: &str, destination: PathBuf) {
    fs::copy(fixture_path(name), destination).expect("copy format fixture");
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/formats")
        .join(name)
}
