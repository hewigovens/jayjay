//! Thin adapter over jayjay-core's app CLI dispatcher: writes the outcome and picks the exit code; command behavior lives in `jayjay_core::cli`.

/// Must run before any GPUI/window init; `None` means "not a CLI command" and the caller falls through to normal window init.
pub fn run_and_exit_if_needed(arguments: &[String]) -> Option<i32> {
    let outcome = jayjay_core::run_app_cli_command(arguments, env!("CARGO_PKG_VERSION"))?;
    if outcome.is_error() {
        eprint!("{}", outcome.message);
    } else {
        print!("{}", outcome.message);
    }
    Some(outcome.exit_code)
}
