// Runs the compiled jayjay-gpui binary as a subprocess with DISPLAY/WAYLAND_DISPLAY unset, proving CLI dispatch exits before any GPUI/window init — required for headless Linux CI.

use std::path::Path;
use std::process::{Command, Output};
use std::sync::Mutex;

use jj_test::LinearFixture;

// JAYJAY_REVIEW_STORE_PATH is a process-wide env var; serialize tests in this binary so they don't race each other's overrides.
static STORE_ENV_LOCK: Mutex<()> = Mutex::new(());

fn run_cli(repo: &Path, store: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jayjay-gpui"))
        .args(args)
        .current_dir(repo)
        .env("JAYJAY_REVIEW_STORE_PATH", store)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .expect("run jayjay-gpui")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf8 stderr")
}

#[test]
fn review_notes_round_trip_matches_core_output_shape() {
    let _lock = STORE_ENV_LOCK.lock().expect("store env lock");
    let fixture = LinearFixture::build();
    let store = fixture.path.with_file_name("review_store.json");

    let add = run_cli(
        &fixture.path,
        &store,
        &[
            "review",
            "add-note",
            "--file",
            "wip1.txt",
            "--line",
            "1",
            "-m",
            "check this",
        ],
    );
    assert!(add.status.success(), "add-note failed: {}", stderr(&add));
    assert!(
        stdout(&add).starts_with("Added review note "),
        "{}",
        stdout(&add)
    );

    let notes_json = run_cli(
        &fixture.path,
        &store,
        &["review", "notes", "--format", "json"],
    );
    assert!(notes_json.status.success());
    let json: serde_json::Value = serde_json::from_str(&stdout(&notes_json)).expect("valid json");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["notes"][0]["status"], "current");
    assert_eq!(json["notes"][0]["note"]["body"], "check this");
    let note_id = json["notes"][0]["note"]["id"]
        .as_str()
        .expect("note id")
        .to_string();

    let resolve = run_cli(&fixture.path, &store, &["review", "resolve-note", &note_id]);
    assert!(resolve.status.success());
    assert_eq!(
        stdout(&resolve),
        format!("Resolved review note {note_id}\n")
    );

    let notes_text = run_cli(&fixture.path, &store, &["review", "notes"]);
    assert!(notes_text.status.success());
    assert_eq!(stdout(&notes_text), "No review notes.\n");

    let notes_resolved = run_cli(
        &fixture.path,
        &store,
        &["review", "notes", "--format", "json", "--include-resolved"],
    );
    let json: serde_json::Value =
        serde_json::from_str(&stdout(&notes_resolved)).expect("valid json");
    assert_eq!(json["notes"][0]["status"], "resolved");
}

#[test]
fn unknown_review_subcommand_exits_nonzero_without_display() {
    let _lock = STORE_ENV_LOCK.lock().expect("store env lock");
    let fixture = LinearFixture::build();
    let store = fixture.path.with_file_name("review_store.json");

    let output = run_cli(&fixture.path, &store, &["review", "bogus"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output), "error: unknown review subcommand: bogus\n");
}

#[test]
fn add_note_rejects_an_unchanged_line_with_repo_error() {
    let _lock = STORE_ENV_LOCK.lock().expect("store env lock");
    let fixture = LinearFixture::build();
    let store = fixture.path.with_file_name("review_store.json");

    let output = run_cli(
        &fixture.path,
        &store,
        &[
            "review", "add-note", "--file", "wip1.txt", "--line", "99", "-m", "nope",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).starts_with("error: "),
        "{}",
        stderr(&output)
    );
}

#[test]
fn version_flag_exits_zero_without_display() {
    let output = Command::new(env!("CARGO_BIN_EXE_jayjay-gpui"))
        .arg("--version")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .expect("run jayjay-gpui --version");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        format!("jayjay {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn config_command_prints_tool_definition_without_display() {
    let output = Command::new(env!("CARGO_BIN_EXE_jayjay-gpui"))
        .arg("config")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .expect("run jayjay-gpui config");

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), jayjay_core::JJ_TOOL_CONFIG);
}
