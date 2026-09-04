//! Palette `jj` commands run with captured pipes, so `run_in_path` forces a
//! non-interactive editor; an editor-requiring command must fail fast, not hang.

use jayjay_core::JjCommand;
use jj_test::init_jj_repo;

#[test]
fn editor_command_fails_fast_instead_of_hanging() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    // `describe` without -m opens the editor; must return non-zero, not hang.
    let result = JjCommand::new("describe")
        .run_in_path(&repo_path)
        .expect("describe should return, not hang");

    assert_ne!(result.exit_code, 0, "editor-requiring command must fail");
    assert!(
        result.output.to_lowercase().contains("editor")
            || result.output.to_lowercase().contains("description"),
        "error should mention the editor/description, got: {}",
        result.output
    );
}

#[test]
fn non_editor_command_still_succeeds() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    let result = JjCommand::new("status")
        .run_in_path(&repo_path)
        .expect("status should run");

    assert_eq!(
        result.exit_code, 0,
        "status must succeed: {}",
        result.output
    );
}

#[test]
fn describe_with_message_does_not_need_editor() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    // -m supplies the message, so no editor is launched.
    let result = JjCommand::new(r#"describe -m "set via palette""#)
        .run_in_path(&repo_path)
        .expect("describe -m should run");

    assert_eq!(
        result.exit_code, 0,
        "describe -m must succeed: {}",
        result.output
    );
}
