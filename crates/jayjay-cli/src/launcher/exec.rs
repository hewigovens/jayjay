mod platform;

use std::ffi::OsString;
use std::process::Command;

use super::{app, gpui};

// Falls back to the GPUI binary when no macOS app bundle is found — the only option on Linux, and useful on dev checkouts that only built the GPUI shell.
pub(crate) fn exec_app<I>(arguments: I) -> !
where
    I: IntoIterator<Item = OsString>,
{
    let Some(executable) = app::find_app_executable().or_else(gpui::find_gpui_executable) else {
        eprintln!("error: JayJay executable not found");
        eprintln!("Install JayJay.app, or build the GPUI shell with: just gpui");
        std::process::exit(127);
    };

    let mut command = Command::new(&executable);
    command.args(arguments);
    let error = platform::exec(command);

    eprintln!("error: failed to run {}: {error}", executable.display());
    std::process::exit(127);
}
