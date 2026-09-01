// Runs the compiled jayjay-gpui binary with DISPLAY/WAYLAND_DISPLAY unset, proving CLI dispatch exits before any GPUI/window init — required for headless Linux CI. Command behavior itself is proven in jayjay-core's cli tests.

use std::process::{Command, Output};

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jayjay-gpui"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .expect("run jayjay-gpui")
}

#[test]
fn version_flag_exits_zero_without_display() {
    let output = run_cli(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("jayjay {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn config_command_prints_tool_definition_without_display() {
    let output = run_cli(&["config"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        jayjay_core::JJ_TOOL_CONFIG
    );
}

#[test]
fn review_errors_reach_stderr_and_exit_nonzero_without_display() {
    let output = run_cli(&["review", "bogus"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: unknown review subcommand: bogus\n"
    );
}
